use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ExtraSpacing;

/// Returns true if `line` has a bare `=` at byte column `col`
/// (excluding compound operators `!=`, `<=`, `>=`, `==`, `=>`).
fn has_eq_at_col(line: &str, col: usize) -> bool {
    let bytes = line.as_bytes();
    if col >= bytes.len() || bytes[col] != b'=' {
        return false;
    }
    let prev = if col > 0 { bytes[col - 1] } else { 0 };
    let next = if col + 1 < bytes.len() { bytes[col + 1] } else { 0 };
    // Exclude compound operators: !=, <=, >=, ==, =>
    prev != b'!' && prev != b'<' && prev != b'>' && prev != b'='
        && next != b'=' && next != b'>'
}

/// Returns true if `line` has `ch` at byte column `col`.
fn has_char_at_col(line: &str, col: usize, ch: u8) -> bool {
    let bytes = line.as_bytes();
    col < bytes.len() && bytes[col] == ch
}

/// Extracts the heredoc terminator word from a line that opens a heredoc (`<<`, `<<-`, `<<~`).
/// Returns `None` if no heredoc is opened on this line.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i + 1 < len {
        match in_str {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(d) if bytes[i] == d => { in_str = None; i += 1; continue; }
            Some(_) => { i += 1; continue; }
            None if bytes[i] == b'"' || bytes[i] == b'\'' => { in_str = Some(bytes[i]); i += 1; continue; }
            None if bytes[i] == b'#' => break,
            None => {}
        }
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') { j += 1; }
            if j < len && matches!(bytes[j], b'\'' | b'"' | b'`') { j += 1; }
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }
        i += 1;
    }
    None
}

/// Returns true if `line` is blank or a pure-comment line (after stripping indent).
fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Returns the absolute byte column where the second token (value) starts on `line`.
/// "Second token" means: skip leading whitespace, skip the first word/token
/// (non-space chars), skip any whitespace, return the column of the next non-space char.
/// Returns `None` if the line has only one token, is blank, or is a comment.
fn value_start_col(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    // Skip leading indent
    while i < len && bytes[i] == b' ' {
        i += 1;
    }
    if i == len || bytes[i] == b'#' {
        return None;
    }
    // Skip the first token (non-space chars)
    while i < len && bytes[i] != b' ' {
        i += 1;
    }
    if i == len {
        return None;
    }
    // Skip spaces between tokens
    while i < len && bytes[i] == b' ' {
        i += 1;
    }
    if i < len && bytes[i] != b'#' {
        Some(i)
    } else {
        None
    }
}

/// Returns a slice containing the first token (non-space chars after leading indent).
fn first_token(line: &str) -> &str {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len && bytes[i] == b' ' {
        i += 1;
    }
    let start = i;
    while i < len && bytes[i] != b' ' {
        i += 1;
    }
    &line[start..i]
}

/// Returns true if `line` has a run of 2+ consecutive non-trailing spaces that suggests
/// it's an alignment peer for a line that has a gap starting at `gap_start_col` and whose
/// value token is at `value_col`.
///
/// A peer is detected if:
/// - The run ENDS within `radius` bytes of `value_col` (same value column = same alignment target)
///   AND the run STARTS at a DIFFERENT position than `gap_start_col` (different padding = truly
///   aligning rather than identical mistake).
/// - OR the run STARTS within `radius` bytes of `gap_start_col` AND its width differs from
///   (value_col - gap_start_col) (different gap size = different identifier length, alignment).
fn has_double_space_near(line: &str, gap_start_col: usize, value_col: usize, radius: usize) -> bool {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let our_gap_width = value_col.saturating_sub(gap_start_col);
    let mut pos = 0;
    while pos < len {
        if bytes[pos] == b' ' && pos + 1 < len && bytes[pos + 1] == b' ' {
            let start = pos;
            let mut end = pos;
            while end < len && bytes[end] == b' ' {
                end += 1;
            }
            // Only consider non-trailing runs
            if end < len {
                let peer_gap_width = end - start;
                let end_near = (end as i64 - value_col as i64).unsigned_abs() as usize <= radius;
                let start_near = (start as i64 - gap_start_col as i64).unsigned_abs() as usize <= radius;
                if end_near && start != gap_start_col {
                    // Same value column but gap starts at a different position → true alignment
                    return true;
                }
                if start_near && peer_gap_width != our_gap_width {
                    // Gap starts nearby but has different width → different identifier lengths,
                    // both padding to align their values
                    return true;
                }
            }
            pos = end;
        } else {
            pos += 1;
        }
    }
    false
}

/// Returns true if a nearby line (other than `current_idx`) has a bare `=` at
/// absolute column `eq_col`.
///
/// Scanning strategy: scan outward from `current_idx`, skipping blank/comment lines and
/// lines with GREATER indent (they are in nested blocks). Stop when we hit a line with
/// LESS indent (we've exited the containing block) or exceed `max_neighbours` same-level
/// non-blank lines. This allows detecting alignment groups that span nested blocks.
fn eq_aligned_nearby(lines: &[&str], current_idx: usize, eq_col: usize, max_neighbours: usize) -> bool {
    let n = lines.len();
    let current_indent = {
        let l = lines[current_idx];
        l.len() - l.trim_start().len()
    };

    // Scan in one direction. `dir` is +1 (forward) or -1 (backward).
    let scan = |dir: i64| -> bool {
        let mut idx = current_idx as i64 + dir;
        let mut non_blank_seen = 0;
        while idx >= 0 && idx < n as i64 {
            let line = lines[idx as usize];
            if is_blank_or_comment(line) {
                idx += dir;
                continue;
            }
            let line_indent = line.len() - line.trim_start().len();
            if line_indent < current_indent {
                break; // Exited the containing block — stop scanning.
            }
            if line_indent > current_indent {
                // Inside a nested block — skip without counting.
                idx += dir;
                continue;
            }
            // Same indent level.
            non_blank_seen += 1;
            if has_eq_at_col(line, eq_col) {
                return true;
            }
            if non_blank_seen >= max_neighbours {
                break;
            }
            idx += dir;
        }
        false
    };

    scan(1) || scan(-1)
}

/// Returns true if `line` has a run of 2+ consecutive spaces AFTER the indent
/// and before absolute byte column `col` (i.e., an interior double-space gap).
/// Leading whitespace (indent) is excluded to avoid false matches.
fn line_has_double_space_before_col(line: &str, col: usize) -> bool {
    let bytes = line.as_bytes();
    let len = bytes.len();
    // Skip leading indent
    let mut i = 0;
    while i < len && bytes[i] == b' ' {
        i += 1;
    }
    let limit = col.min(len);
    // Check for double-space after indent and before col
    while i + 1 < limit {
        if bytes[i] == b' ' && bytes[i + 1] == b' ' {
            return true;
        }
        i += 1;
    }
    false
}

impl Rule for ExtraSpacing {
    fn name(&self) -> &'static str {
        "Layout/ExtraSpacing"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        // Track heredoc state: when we enter a heredoc body we skip until the terminator.
        let mut heredoc_terminator: Option<String> = None;

        for (i, line) in lines.iter().enumerate() {
            // If inside a heredoc body, check if this line is the terminator.
            if let Some(ref term) = heredoc_terminator.clone() {
                if line.trim() == term.as_str() {
                    heredoc_terminator = None;
                }
                continue; // Skip heredoc body lines entirely.
            }

            // Skip indentation (leading whitespace)
            let indent_len = line.len() - line.trim_start().len();
            let content = &line[indent_len..];

            // Skip pure comment lines
            let content_trimmed = content.trim_start();
            if content_trimmed.starts_with('#') {
                continue;
            }

            // Scan for consecutive spaces outside strings
            let bytes = content.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            // Tracks nesting depth of `%(...)` percent-literal strings.
            // When > 0, we are inside a percent string and should not flag spacing.
            let mut paren_depth: u32 = 0;
            let mut j = 0;
            while j < len {
                let b = bytes[j];
                // Handle percent-string delimiters `%(`, `)` nesting.
                if in_string.is_none() {
                    if paren_depth > 0 {
                        if b == b'\\' { j += 2; continue; }
                        if b == b'(' { paren_depth += 1; j += 1; continue; }
                        if b == b')' {
                            paren_depth -= 1;
                            j += 1;
                            continue;
                        }
                        j += 1;
                        continue;
                    }
                    // Detect `%(` — Ruby percent string with parens.
                    // We check for `%` followed by optional type letter then `(`.
                    if b == b'%' && j + 1 < len {
                        let next = bytes[j + 1];
                        if next == b'(' {
                            paren_depth = 1;
                            j += 2;
                            continue;
                        }
                        // %w( %i( %q( %Q( etc.
                        if next.is_ascii_alphabetic() && j + 2 < len && bytes[j + 2] == b'(' {
                            paren_depth = 1;
                            j += 3;
                            continue;
                        }
                    }
                }
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment
                    None => {}
                }
                // Check for two or more consecutive spaces
                if b == b' ' && j + 1 < len && bytes[j + 1] == b' ' {
                    let span_start = j;
                    while j < len && bytes[j] == b' ' {
                        j += 1;
                    }
                    let span_end = j;

                    // Skip trailing whitespace — all bytes from span_start to end of
                    // content are spaces, meaning these spaces run to EOL. TrailingWhitespace
                    // handles that case; ExtraSpacing must not double-report it.
                    let rest_all_spaces = bytes[span_end..].iter().all(|&b| b == b' ');
                    if rest_all_spaces {
                        j = span_end;
                        continue;
                    }

                    // Skip alignment spacing after comma — e.g. `[10..10,   0..255]`.
                    // Extra spaces immediately following a `,` are valid Ruby column-alignment style.
                    if span_start > 0 && bytes[span_start - 1] == b',' {
                        j = span_end;
                        continue;
                    }

                    // Skip extra spaces immediately before a `#` comment (comment-alignment).
                    if span_end < len && bytes[span_end] == b'#' {
                        j = span_end;
                        continue;
                    }

                    // Skip extra spaces that follow a heredoc opener (`<<~`, `<<-`, `<<IDENT`).
                    // Ruby allows `fail <<~EOM  if condition` and similar patterns where a
                    // trailing modifier follows the heredoc delimiter with extra spaces.
                    // Rubocop does not flag double-spaces in this position.
                    {
                        let prefix = &content[..span_start];
                        if prefix.contains("<<") {
                            j = span_end;
                            continue;
                        }
                    }

                    // Absolute column of the first non-space token after the gap.
                    let token_col = indent_len + span_end;

                    // Skip column-aligned `=` (e.g., vertically-aligned assignments).
                    // We look at the nearest non-blank, non-comment neighbours (up to 3 in
                    // each direction, skipping blank lines and comments) so that constants
                    // separated by comment blocks are handled correctly.
                    if span_end < len && bytes[span_end] == b'=' {
                        let after_eq = if span_end + 1 < len { bytes[span_end + 1] } else { 0 };
                        // Only treat as alignment `=`, not compound operators `==`, `=>`
                        if after_eq != b'=' && after_eq != b'>' {
                            if eq_aligned_nearby(lines, i, token_col, 10) {
                                j = span_end;
                                continue;
                            }
                        }
                    }

                    // Skip compound assignment operators (`||=`, `&&=`, `+=`, etc.) that are
                    // aligned across lines. The leading `|`/`&`/`+` won't look like `=`, so
                    // find where the `=` of the compound op falls and check nearby lines.
                    if span_end < len {
                        let tc = bytes[span_end];
                        let compound_eq_col: Option<usize> = if tc == b'|'
                            && span_end + 2 < len
                            && bytes[span_end + 1] == b'|'
                            && bytes[span_end + 2] == b'='
                        {
                            Some(token_col + 2)
                        } else if tc == b'&'
                            && span_end + 2 < len
                            && bytes[span_end + 1] == b'&'
                            && bytes[span_end + 2] == b'='
                        {
                            Some(token_col + 2)
                        } else if (tc == b'+' || tc == b'-' || tc == b'*'
                            || tc == b'/' || tc == b'%')
                            && span_end + 1 < len
                            && bytes[span_end + 1] == b'='
                        {
                            Some(token_col + 1)
                        } else {
                            None
                        };
                        if let Some(eq_col) = compound_eq_col {
                            if eq_aligned_nearby(lines, i, eq_col, 10) {
                                j = span_end;
                                continue;
                            }
                        }
                    }

                    // Skip general column-alignment: any non-space token aligned across
                    // adjacent lines (e.g. `after_create_commit  :foo` / `after_destroy_commit :foo`).
                    // Expanded to ±3 lines (was ±2).
                    if span_end < len && bytes[span_end] != b' ' {
                        let target_char = bytes[span_end];
                        let is_aligned = (i > 0 && has_char_at_col(&lines[i - 1], token_col, target_char))
                            || (i + 1 < lines.len() && has_char_at_col(&lines[i + 1], token_col, target_char))
                            || (i > 1 && has_char_at_col(&lines[i - 2], token_col, target_char))
                            || (i + 2 < lines.len() && has_char_at_col(&lines[i + 2], token_col, target_char))
                            || (i > 2 && has_char_at_col(&lines[i - 3], token_col, target_char))
                            || (i + 3 < lines.len() && has_char_at_col(&lines[i + 3], token_col, target_char));
                        if is_aligned {
                            j = span_end;
                            continue;
                        }
                    }

                    // Skip alignment groups where adjacent lines align their values to
                    // the same column — e.g. Fabricator DSL blocks where longer field names
                    // use single space (`last_read_id 0`) while shorter ones pad with extra
                    // spaces (`timeline     'home'`), both placing the value at the same column.
                    //
                    // Also handle the RSpec `let` pattern where the same method name is used
                    // in two nearby `let` declarations with different spacing:
                    //   `let(:path) { '/api/v1/accounts' }` (single-space)
                    //   `let(:path)  { '/api/v1/accounts.json' }` (double-space)
                    //
                    // Heuristic: if any adjacent non-blank, non-comment line at the SAME indent
                    // level satisfies one of these conditions, treat as an alignment group:
                    //   1. Has a double-space run near our gap-start or value-start column (±2).
                    //   2. Has its value starting at exactly our token_col (same-column anchor).
                    //   3. Shares the SAME first token as the current line (same method/DSL call).
                    {
                        let our_gap_col = indent_len + span_start;
                        let our_first_token = first_token(line);
                        let mut found_peer = false;
                        'peer: for delta in 1..=8usize {
                            for &dir in &[-1i64, 1i64] {
                                let ni = i as i64 + dir * delta as i64;
                                if ni < 0 || ni >= lines.len() as i64 {
                                    continue;
                                }
                                let ni = ni as usize;
                                let neighbor = &lines[ni];
                                if is_blank_or_comment(neighbor) {
                                    continue;
                                }
                                let n_indent = neighbor.len() - neighbor.trim_start().len();
                                if n_indent != indent_len {
                                    continue;
                                }
                                // Check 1: neighbor also has double-spaces near our gap or value col
                                if has_double_space_near(neighbor, our_gap_col, token_col, 2) {
                                    found_peer = true;
                                    break 'peer;
                                }
                                // Check 2: neighbor's value starts at exactly our token_col
                                // (covers single-space neighbors in the same alignment group).
                                // Only count as peer if the neighbor doesn't itself have extra
                                // spaces before that column (otherwise both lines are wrong and
                                // are not truly aligned to each other).
                                if value_start_col(neighbor) == Some(token_col)
                                    && !line_has_double_space_before_col(neighbor, token_col)
                                {
                                    found_peer = true;
                                    break 'peer;
                                }
                                // Check 3: neighbor starts with the same first token AND
                                // it looks like a method call (contains `(` or is long enough).
                                // This covers `let(:path)  {` next to `let(:path) {` and
                                // `fail_with_message  '...'` next to `fail_with_message '...'`.
                                // We require len >= 4 and the token to contain `(` or be >= 8 chars
                                // to avoid incorrectly skipping short variable assignments like
                                // `x  = 1` / `x = 2`.
                                let nft = first_token(neighbor);
                                if !our_first_token.is_empty()
                                    && nft == our_first_token
                                    && (our_first_token.contains('(')
                                        || our_first_token.len() >= 8)
                                {
                                    found_peer = true;
                                    break 'peer;
                                }
                            }
                        }
                        if found_peer {
                            j = span_end;
                            continue;
                        }
                    }

                    // Last resort: file-wide same-first-token search.
                    // If any other line in the file (any indent) starts with the same
                    // method-call-like token, treat this as a potential alignment group.
                    // For tokens that look like DSL calls with symbol arguments (`let(:`,
                    // `subject(:`, `context(:`, etc.), use prefix matching (any other `let(`
                    // call in the file counts), because RSpec files often have multiple `let`
                    // declarations with different symbol names that are all aligned.
                    // For other method-call tokens (containing `(` or length ≥ 8), require
                    // exact token match.
                    {
                        let our_first_token = first_token(line);
                        let is_dsl_call = our_first_token.contains("(:"); // e.g. let(:user)
                        let is_method_call = !is_dsl_call
                            && (our_first_token.contains('(') || our_first_token.len() >= 8);
                        if is_dsl_call || is_method_call {
                            // For DSL calls like `let(:user)`, extract base name `let(`
                            let match_prefix: &str = if is_dsl_call {
                                if let Some(idx) = our_first_token.find("(:") {
                                    &our_first_token[..=idx] // e.g. "let("
                                } else {
                                    our_first_token
                                }
                            } else {
                                our_first_token
                            };
                            let file_wide_peer = lines.iter().enumerate().any(|(li, l)| {
                                li != i
                                    && !is_blank_or_comment(l)
                                    && if is_dsl_call {
                                        first_token(l).starts_with(match_prefix)
                                    } else {
                                        first_token(l) == our_first_token
                                    }
                            });
                            if file_wide_peer {
                                j = span_end;
                                continue;
                            }
                        }
                    }

                    let line_start = ctx.line_start_offsets[i] as usize;
                    let abs_start = (line_start + indent_len + span_start) as u32;
                    let abs_end = (line_start + indent_len + span_end) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra spacing detected.".into(),
                        range: TextRange::new(abs_start, abs_end),
                        severity: Severity::Warning,
                    });
                    continue;
                }
                j += 1;
            }

            // After scanning this line, check if it opens a heredoc for the next lines.
            if heredoc_terminator.is_none() {
                heredoc_terminator = extract_heredoc_terminator(line);
            }
        }

        diags
    }
}

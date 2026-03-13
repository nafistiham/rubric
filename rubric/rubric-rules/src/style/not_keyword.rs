use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NotKeyword;

impl Rule for NotKeyword {
    fn name(&self) -> &'static str {
        "Style/Not"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        // Track heredoc state: when Some(term), we are inside a heredoc body
        // and skip all lines until we see a line whose trimmed content equals `term`.
        let mut in_heredoc: Option<String> = None;

        for (i, line) in lines.iter().enumerate() {
            // Handle heredoc body/end
            if let Some(ref terminator) = in_heredoc.clone() {
                if line.trim() == terminator.as_str() {
                    in_heredoc = None;
                }
                continue; // skip heredoc body lines (including the terminator line itself)
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect heredoc opener on this line before scanning for `not`.
            // We detect it here so the opener line itself (e.g. `abort <<~ERROR`) is
            // still scanned for `not` occurrences before the heredoc marker.
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: still scan the opener line for `not` below.
            }

            let bytes = line.as_bytes();
            let len = bytes.len();
            let pattern = b"not";
            let pat_len = pattern.len();

            // `in_literal`: tracks what kind of literal we are inside.
            //   None          — not inside any literal
            //   Some(b'"')    — inside double-quoted string
            //   Some(b'\'')   — inside single-quoted string
            //   Some(b'/')    — inside regex literal /pattern/
            let mut in_literal: Option<u8> = None;

            // Track last non-space byte seen (for regex disambiguation)
            let mut last_non_space: u8 = b' ';

            let mut j = 0;
            while j < len {
                let b = bytes[j];

                match in_literal {
                    Some(_) if b == b'\\' => {
                        // Escape: skip next byte
                        j += 2;
                        continue;
                    }
                    Some(delim) if b == delim => {
                        // Closing delimiter
                        in_literal = None;
                        last_non_space = b;
                        j += 1;
                        continue;
                    }
                    Some(_) => {
                        j += 1;
                        continue;
                    }
                    None if b == b'"' || b == b'\'' => {
                        in_literal = Some(b);
                        last_non_space = b;
                        j += 1;
                        continue;
                    }
                    None if b == b'/' => {
                        // Determine if this `/` opens a regex literal.
                        // A regex-starting `/` appears after an operator, open delimiter,
                        // comma, or at the start of an expression — not after an identifier
                        // or closing delimiter (where it would be division).
                        if is_regex_start(last_non_space) {
                            in_literal = Some(b'/');
                            last_non_space = b'/';
                            j += 1;
                            continue;
                        }
                        // Otherwise it's division — fall through and record as last_non_space
                        last_non_space = b;
                        j += 1;
                        continue;
                    }
                    None if b == b'#' => break, // comment — stop scanning this line
                    // Percent literals: %r!...!, %(str), %w(...) etc.
                    // Skip the entire literal content so `not` inside is not flagged.
                    None if b == b'%' && j + 1 < len => {
                        let mut k = j + 1;
                        // Skip optional type sigil (r, q, Q, w, W, i, I, s, x)
                        if k < len && bytes[k].is_ascii_alphabetic() { k += 1; }
                        if k < len {
                            let open = bytes[k];
                            let close = match open {
                                b'(' => b')',
                                b'[' => b']',
                                b'{' => b'}',
                                b'<' => b'>',
                                c if c.is_ascii_punctuation() => c, // same-char
                                _ => { last_non_space = b; j += 1; continue; }
                            };
                            k += 1; // skip opening delimiter
                            if open == close {
                                // Same-char delimiter: scan until unescaped close
                                while k < len {
                                    if bytes[k] == b'\\' { k += 2; continue; }
                                    if bytes[k] == close { k += 1; break; }
                                    k += 1;
                                }
                            } else {
                                // Bracket-style: depth-tracked
                                let mut depth = 1usize;
                                while k < len && depth > 0 {
                                    if bytes[k] == b'\\' { k += 2; continue; }
                                    if bytes[k] == open { depth += 1; }
                                    else if bytes[k] == close { depth -= 1; }
                                    k += 1;
                                }
                            }
                            j = k;
                            last_non_space = close;
                            continue;
                        }
                        last_non_space = b; j += 1; continue;
                    }
                    None => {}
                }

                // Check for `not` keyword
                if j + pat_len <= len && &bytes[j..j + pat_len] == pattern {
                    // Word boundary before — also exclude `.` so `.not(` (method call) is skipped
                    let before_ok = j == 0
                        || (!bytes[j - 1].is_ascii_alphanumeric()
                            && bytes[j - 1] != b'_'
                            && bytes[j - 1] != b'.');
                    // Word boundary after
                    let after_pos = j + pat_len;
                    let after_ok = after_pos >= len
                        || (!bytes[after_pos].is_ascii_alphanumeric() && bytes[after_pos] != b'_');

                    if before_ok && after_ok {
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Use `!` instead of `not` keyword.".into(),
                            range: TextRange::new(
                                line_start + j as u32,
                                line_start + (j + pat_len) as u32,
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }

                if b != b' ' && b != b'\t' {
                    last_non_space = b;
                }
                j += 1;
            }
        }

        diags
    }
}

/// Returns true if the character `last` (the last non-space byte before a `/`)
/// indicates that the `/` opens a regex literal rather than being a division operator.
///
/// A `/` is NOT regex only after a digit, a closing bracket/paren/brace, or a
/// closing string quote — all of which indicate the preceding expression has a
/// value that `/` would divide.  After identifiers (method names, variables),
/// operators, or opening delimiters, `/` is treated as a regex opener.  This
/// matches the heuristic most Ruby text scanners use and avoids flagging `not`
/// inside regex patterns like `Then /^output should not contain.../`.
fn is_regex_start(last: u8) -> bool {
    !matches!(last, b'0'..=b'9' | b')' | b']' | b'}' | b'"' | b'\'')
}

/// Extract the heredoc terminator word from a line containing `<<~TERM`, `<<-TERM`, or `<<TERM`.
/// Returns `None` if no heredoc opener is found on this line.
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
            // Skip optional `-` or `~`
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') {
                j += 1;
            }
            // Skip optional quote around terminator
            if j < len && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`') {
                j += 1;
            }
            // Read the terminator word
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }

        i += 1;
    }

    None
}

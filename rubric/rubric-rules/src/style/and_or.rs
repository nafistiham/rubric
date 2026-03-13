use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AndOr;

const FLOW_CONTROL_KEYWORDS: &[&str] = &["raise", "return", "next", "break", "fail"];

fn is_flow_control_rhs(rest: &str) -> bool {
    let word = rest.trim_start();
    FLOW_CONTROL_KEYWORDS.iter().any(|&kw| {
        word.starts_with(kw)
            && word[kw.len()..]
                .chars()
                .next()
                .map_or(true, |c| !c.is_alphanumeric() && c != '_')
    })
}

/// Lexical context at a given byte position on a line.
#[derive(PartialEq)]
enum PosContext {
    /// Plain code — the position may contain `and`/`or` as keywords.
    Code,
    /// Inside a string, regex, percent-literal, or comment — skip.
    NonCode,
}

/// Scan `line` up to (but not including) `pos` and return whether that
/// position is inside code or a non-code region (string, regex literal,
/// percent literal, or inline comment).
///
/// Handles:
///   - Double- and single-quoted strings with backslash escapes.
///   - Regex literals `/pattern/` with a simple heuristic: a `/` that
///     follows an operator-like character (`=`, `(`, `,`, `[`, `{`,
///     whitespace at line start, `!`, `~`, `&`, `|`, `<`, `>`) opens a
///     regex.  A `/` that follows a word character closes/divides.
///   - Percent literals `%q(...)`, `%Q(...)`, `%(...)`, `%w[...]`,
///     `%i[...]`, `%r{...}`, `%x(...)`, `%<...>` (any bracket pair).
///   - Inline comments: a bare `#` outside any string/literal stops code.
fn pos_context(line: &[u8], pos: usize) -> PosContext {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        InDoubleStr,
        InSingleStr,
        InRegex,
        InPercent(u8, u8), // open_char, close_char — depth tracked separately
        InComment,
    }

    let mut state = State::Code;
    let mut percent_depth: u32 = 0;
    let mut i = 0;
    let lim = pos.min(line.len());

    // Track what the last non-whitespace byte was (for regex heuristic).
    let mut last_non_ws: u8 = b';'; // pretend we're at statement start

    while i < lim {
        let b = line[i];
        match state {
            State::InComment => {
                // Everything from here to `pos` is a comment.
                return PosContext::NonCode;
            }
            State::InSingleStr => {
                if b == b'\\' {
                    i += 2;
                    continue;
                } else if b == b'\'' {
                    state = State::Code;
                }
            }
            State::InDoubleStr => {
                if b == b'\\' {
                    i += 2;
                    continue;
                } else if b == b'#' && i + 1 < lim && line[i + 1] == b'{' {
                    // Skip #{...} interpolation — track brace depth so that
                    // `"` chars inside the interpolation don't close the string.
                    i += 2; // skip `#` and `{`
                    let mut depth = 1usize;
                    while i < lim && depth > 0 {
                        if line[i] == b'\\' { i += 2; continue; }
                        if line[i] == b'{' { depth += 1; }
                        else if line[i] == b'}' { depth -= 1; }
                        i += 1;
                    }
                    continue;
                } else if b == b'"' {
                    state = State::Code;
                }
            }
            State::InRegex => {
                if b == b'\\' {
                    i += 2;
                    continue;
                } else if b == b'/' {
                    state = State::Code;
                }
            }
            State::InPercent(open, close) => {
                if b == b'\\' {
                    i += 2;
                    continue;
                } else if b == close {
                    if percent_depth == 0 {
                        state = State::Code;
                    } else {
                        percent_depth -= 1;
                    }
                } else if b == open && open != close {
                    percent_depth += 1;
                }
            }
            State::Code => {
                match b {
                    b'\'' => state = State::InSingleStr,
                    b'"' => state = State::InDoubleStr,
                    b'#' => {
                        // `#{` is interpolation inside a double string —
                        // but we already handle that in InDoubleStr.
                        // A bare `#` in code context starts a comment.
                        state = State::InComment;
                        // Everything after here is a comment; if pos > i we
                        // will return NonCode on the next iteration.
                    }
                    b'/' => {
                        // Regex heuristic: `/` opens a regex when the previous
                        // meaningful token looks like an operator context.
                        if is_regex_opener(last_non_ws) {
                            state = State::InRegex;
                        }
                        // else: division operator — stay in Code
                    }
                    b'%' => {
                        // Percent literal: %q(...) %w[...] %( etc.
                        // Peek ahead to find the sigil character and bracket.
                        let j = i + 1;
                        if j < lim {
                            // Optional letter sigil (q, Q, w, W, i, I, r, x, s)
                            let (sigil_len, brace_pos) = if line[j].is_ascii_alphabetic() {
                                (1usize, j + 1)
                            } else {
                                (0usize, j)
                            };
                            let _ = sigil_len;
                            if brace_pos < lim {
                                if let Some(close) = matching_close(line[brace_pos]) {
                                    state = State::InPercent(line[brace_pos], close);
                                    percent_depth = 0;
                                    i = brace_pos + 1;
                                    continue;
                                }
                            }
                        }
                        // Not a valid percent literal opener — treat as operator.
                    }
                    _ => {}
                }
                if !b.is_ascii_whitespace() {
                    last_non_ws = b;
                }
            }
        }
        i += 1;
    }

    match state {
        State::Code => PosContext::Code,
        _ => PosContext::NonCode,
    }
}

/// Returns true if a `/` following `last_non_ws` should be treated as a
/// regex opener rather than a division operator.
fn is_regex_opener(last: u8) -> bool {
    matches!(
        last,
        b'=' | b'(' | b',' | b'[' | b'{' | b'!' | b'~' | b'&' | b'|'
            | b'<' | b'>' | b'+' | b'-' | b'*' | b':' | b';' | b'\n'
    )
}

/// Given a bracket/brace/paren character, return its matching close character.
/// Returns `None` if the character is not a recognised opener.
fn matching_close(open: u8) -> Option<u8> {
    match open {
        b'(' => Some(b')'),
        b'[' => Some(b']'),
        b'{' => Some(b'}'),
        b'<' => Some(b'>'),
        b'|' => Some(b'|'),
        b'/' => Some(b'/'),
        b'!' => Some(b'!'),
        _ => None,
    }
}

impl Rule for AndOr {
    fn name(&self) -> &'static str {
        "Style/AndOr"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        // Track heredoc state
        let mut in_heredoc: Option<String> = None;
        // Track cross-line multiline string/percent-literal state.
        // When Some((close_char, depth)), the current line is inside a multiline
        // percent literal (e.g. %w(...) or %{...}) that spans multiple lines.
        let mut in_multiline_percent: Option<(u8, usize)> = None;
        // Track cross-line multiline double-quoted or single-quoted string.
        // When Some(delim), the current line is inside an unclosed string literal.
        let mut in_multiline_string: Option<u8> = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Handle heredoc end
            if let Some(ref terminator) = in_heredoc.clone() {
                if line.trim() == terminator.as_str() {
                    in_heredoc = None;
                }
                continue; // skip heredoc body lines
            }

            // If inside a multiline double/single-quoted string, scan for close.
            if let Some(delim) = in_multiline_string {
                let bytes = line.as_bytes();
                let mut j = 0;
                let mut closed = false;
                while j < bytes.len() {
                    if bytes[j] == b'\\' { j += 2; continue; }
                    if bytes[j] == delim { closed = true; break; }
                    j += 1;
                }
                if closed { in_multiline_string = None; }
                continue; // skip: all content is inside the string
            }

            // If we're inside a multiline percent literal, scan for the closing
            // delimiter (depth-tracked for bracket delimiters).
            if let Some((close, ref mut depth)) = in_multiline_percent {
                let open = match close { b')' => b'(', b']' => b'[', b'}' => b'{', b'>' => b'<', c => c };
                let bytes = line.as_bytes();
                let mut found_close = false;
                let mut j = 0;
                while j < bytes.len() {
                    if bytes[j] == b'\\' { j += 2; continue; }
                    if bytes[j] == open && open != close { *depth += 1; }
                    else if bytes[j] == close {
                        if *depth == 0 { found_close = true; break; }
                        *depth -= 1;
                    }
                    j += 1;
                }
                if found_close {
                    in_multiline_percent = None;
                    // The rest of the line after the close might have code; but
                    // to be safe we skip the whole line (close is rarely followed
                    // by `and`/`or` in practice).
                }
                continue;
            }

            if trimmed.starts_with('#') {
                continue;
            }

            // Detect heredoc openers: <<~TERM, <<-TERM, <<TERM (quoted or bare)
            // Extract the terminator to know when the heredoc ends.
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Don't skip this line entirely — the content after the opener is code
            }

            let bytes = line.as_bytes();

            // Detect multiline percent literals opened on this line that don't
            // close before end-of-line. We scan for `%`, optional sigil letter,
            // bracket/delimiter char, and then look for the matching close.
            {
                let mut j = 0;
                let mut found_multiline = false;
                let mut in_str: Option<u8> = None;
                while j < bytes.len() {
                    match in_str {
                        Some(_) if bytes[j] == b'\\' => { j += 2; continue; }
                        Some(d) if bytes[j] == d => { in_str = None; j += 1; continue; }
                        Some(_) => { j += 1; continue; }
                        None if bytes[j] == b'"' || bytes[j] == b'\'' => { in_str = Some(bytes[j]); j += 1; continue; }
                        None if bytes[j] == b'#' => break,
                        None => {}
                    }
                    if bytes[j] == b'%' && j + 1 < bytes.len() {
                        let mut k = j + 1;
                        if k < bytes.len() && bytes[k].is_ascii_alphabetic() { k += 1; }
                        if k < bytes.len() {
                            let open = bytes[k];
                            let close = match open {
                                b'(' => b')', b'[' => b']', b'{' => b'}', b'<' => b'>',
                                c if c.is_ascii_punctuation() => c,
                                _ => { j += 1; continue; }
                            };
                            // Scan forward to find the matching close (or end of line).
                            let mut depth = 0usize;
                            let mut m = k + 1;
                            let mut closed = false;
                            while m < bytes.len() {
                                if bytes[m] == b'\\' { m += 2; continue; }
                                if bytes[m] == open && open != close { depth += 1; }
                                else if bytes[m] == close {
                                    if depth == 0 { closed = true; break; }
                                    depth -= 1;
                                }
                                m += 1;
                            }
                            if !closed {
                                // Multiline percent literal — mark and skip subsequent lines.
                                in_multiline_percent = Some((close, depth));
                                found_multiline = true;
                                break;
                            }
                            j = m + 1;
                            continue;
                        }
                    }
                    j += 1;
                }
                if found_multiline {
                    // The opener line itself might have `and`/`or` before the %{,
                    // but after it everything is non-code. To keep it simple and
                    // avoid FPs, skip searching on this line when it opens a
                    // multiline percent literal. Real violations on the opener
                    // line (before the `%`) are rare.
                    continue;
                }
            }

            // If the previous non-empty line ends with `.`, this line may be a method
            // chain continuation (e.g., rspec `.and include(...)`). Detect this so
            // `and`/`or` at the start of the line is treated as a method call, not a keyword.
            let prev_line_ends_with_dot = i > 0 && {
                let prev = lines[i - 1].trim_end();
                prev.ends_with('.')
            };

            for (pattern, kw_len) in &[(" and ", 3usize), (" or ", 2usize)] {
                let mut search_start = 0usize;
                while let Some(pos) = line[search_start..].find(pattern) {
                    let abs_pos = search_start + pos;

                    // Skip if this match is inside a string, comment, regex, or percent literal
                    if pos_context(bytes, abs_pos) != PosContext::Code {
                        search_start = abs_pos + 1;
                        if search_start >= line.len() {
                            break;
                        }
                        continue;
                    }

                    // Skip if `and`/`or` appears at the start of the line (only whitespace
                    // before it) and the previous line ends with `.` — this is a method chain
                    // continuation (e.g., rspec `.and include(...)`), not the keyword.
                    if prev_line_ends_with_dot && bytes[..abs_pos].iter().all(|&b| b == b' ' || b == b'\t') {
                        search_start = abs_pos + 1;
                        if search_start >= line.len() { break; }
                        continue;
                    }

                    let after_pattern = &line[abs_pos + pattern.len()..];

                    if is_flow_control_rhs(after_pattern) {
                        search_start = abs_pos + pattern.len();
                        if search_start >= line.len() {
                            break;
                        }
                        continue;
                    }

                    let kw_start = abs_pos + 1;
                    let line_start = ctx.line_start_offsets[i] as usize;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `&&`/`||` instead of `and`/`or` keyword.".to_string(),
                        range: TextRange::new(
                            (line_start + kw_start) as u32,
                            (line_start + kw_start + kw_len) as u32,
                        ),
                        severity: Severity::Warning,
                    });
                    search_start = abs_pos + pattern.len();
                    if search_start >= line.len() {
                        break;
                    }
                }
            }

            // After processing this line, detect if it ends inside an unclosed
            // double- or single-quoted string. If so, subsequent lines are
            // inside that string and must be skipped.
            {
                let mut str_state: Option<u8> = None;
                let mut j = 0;
                while j < bytes.len() {
                    let b = bytes[j];
                    match str_state {
                        Some(_) if b == b'\\' => { j += 2; continue; }
                        Some(d) if b == d => { str_state = None; }
                        Some(_) => {}
                        None if b == b'"' || b == b'\'' => { str_state = Some(b); }
                        None if b == b'#' => break,
                        None => {}
                    }
                    j += 1;
                }
                in_multiline_string = str_state;
            }
        }

        diags
    }
}

/// Extract the heredoc terminator from a line containing `<<~TERM`, `<<-TERM`, or `<<TERM`.
/// Returns `None` if no heredoc opener found.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let mut i = 0;
    let bytes = line.as_bytes();
    let len = bytes.len();
    // Track string state to avoid matching << inside strings
    let mut in_str: Option<u8> = None;
    while i + 1 < len {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
                i += 1;
                continue;
            }
            Some(_) => {
                i += 1;
                continue;
            }
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
                i += 1;
                continue;
            }
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
            let quote = if j < len
                && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`')
            {
                let q = bytes[j];
                j += 1;
                Some(q)
            } else {
                None
            };
            let _ = quote; // terminator word is what matters
            // Read the terminator word
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j > start {
                let term = &line[start..j];
                return Some(term.to_string());
            }
        }
        i += 1;
    }
    None
}

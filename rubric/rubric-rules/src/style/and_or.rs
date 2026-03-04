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
                } else if b == b'"' {
                    state = State::Code;
                }
                // Note: #{...} interpolation is skipped conservatively —
                // we don't recurse into it; treating it as part of the string
                // is safe because the heredoc guard already handles multi-line.
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
                } else if b == open {
                    percent_depth += 1;
                } else if b == close {
                    if percent_depth == 0 {
                        state = State::Code;
                    } else {
                        percent_depth -= 1;
                    }
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

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Handle heredoc end
            if let Some(ref terminator) = in_heredoc.clone() {
                if line.trim() == terminator.as_str() {
                    in_heredoc = None;
                }
                continue; // skip heredoc body lines
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

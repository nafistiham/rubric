use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CharacterLiteral;

/// Returns true if this byte can precede a `?` that starts a character literal.
/// A `?` starts a char literal when it follows whitespace or punctuation used
/// in expressions, not when it ends a method name (preceded by alphanumeric / `_`).
fn is_char_literal_prefix(b: u8) -> bool {
    matches!(
        b,
        b' ' | b'\t'
            | b'('
            | b'['
            | b'{'
            | b','
            | b'='
            | b'!'
            | b'+'
            | b'-'
            | b'*'
            | b'/'
            | b'%'
            | b'|'
            | b'&'
            | b'^'
            | b'~'
            | b'<'
            | b'>'
    )
}

/// Extract heredoc terminator from a line (e.g. `<<-'TERM'` → `"TERM"`).
fn extract_heredoc_terminator(bytes: &[u8]) -> Option<String> {
    let len = bytes.len();
    let mut i = 0;
    while i + 1 < len {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut k = i + 2;
            if k < len && (bytes[k] == b'-' || bytes[k] == b'~') {
                k += 1;
            }
            let quote = if k < len && (bytes[k] == b'\'' || bytes[k] == b'"' || bytes[k] == b'`') {
                let q = bytes[k];
                k += 1;
                Some(q)
            } else {
                None
            };
            let term_start = k;
            if let Some(q) = quote {
                while k < len && bytes[k] != q {
                    k += 1;
                }
            } else {
                while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                    k += 1;
                }
            }
            if k > term_start {
                return Some(String::from_utf8_lossy(&bytes[term_start..k]).to_string());
            }
        }
        i += 1;
    }
    None
}

impl Rule for CharacterLiteral {
    fn name(&self) -> &'static str {
        "Style/CharacterLiteral"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_multiline_regex = false;
        let mut in_heredoc: Option<String> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // Skip body of multiline /regex/ (e.g. /x flag spanning multiple lines)
            if in_multiline_regex {
                let bytes = line.as_bytes();
                let n = bytes.len();
                let mut j = 0;
                while j < n {
                    match bytes[j] {
                        b'\\' => { j += 2; continue; }
                        b'/' => { in_multiline_regex = false; break; }
                        _ => { j += 1; }
                    }
                }
                continue;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Check for heredoc start before processing the line
            let bytes = line.as_bytes();
            if let Some(term) = extract_heredoc_terminator(bytes) {
                in_heredoc = Some(term);
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut in_percent_literal = false;
            let mut percent_close: u8 = 0;
            let mut in_regex = false;
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                // Inside a /regex/ literal — skip until closing unescaped /
                if in_regex {
                    match b {
                        b'\\' => { j += 2; continue; }
                        b'/' => { in_regex = false; }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                // Inside a %r or other percent literal — skip until closing delimiter
                if in_percent_literal {
                    if b == b'\\' {
                        j += 2;
                        continue;
                    }
                    if b == percent_close {
                        in_percent_literal = false;
                    }
                    j += 1;
                    continue;
                }

                if let Some(delim) = in_string {
                    match b {
                        b'\\' => {
                            j += 2;
                            continue;
                        }
                        c if c == delim => {
                            in_string = None;
                        }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                match b {
                    b'%' if j + 2 < n => {
                        // Detect %r, %w, %i, %W, %I, %q, %Q, %x, %s literals
                        let next = bytes[j + 1];
                        if matches!(next, b'r' | b'w' | b'W' | b'i' | b'I' | b'q' | b'Q' | b'x' | b's') {
                            let open = bytes[j + 2];
                            let close = match open {
                                b'(' => b')',
                                b'[' => b']',
                                b'{' => b'}',
                                b'<' => b'>',
                                _ => open,
                            };
                            in_percent_literal = true;
                            percent_close = close;
                            j += 3;
                            continue;
                        }
                    }
                    b'/' => {
                        // Detect /regex/ — only in operator/start contexts
                        let prev_pos = bytes[..j].iter().rposition(|&c| c != b' ' && c != b'\t');
                        let prev = prev_pos.map(|p| bytes[p]);
                        // `~` covers `=~` and `!~` match operators
                        let mut is_regex = matches!(prev, None
                            | Some(b'(') | Some(b'[') | Some(b',') | Some(b'=')
                            | Some(b'!') | Some(b'|') | Some(b'&') | Some(b'?') | Some(b':')
                            | Some(b'{') | Some(b';') | Some(b'>') | Some(b'<') | Some(b'~'));
                        // Also treat `/` as regex start when preceded by a Ruby keyword
                        if !is_regex {
                            if let Some(p) = prev_pos {
                                if bytes[p].is_ascii_alphabetic() || bytes[p] == b'_' {
                                    let word_start = bytes[..=p].iter().rposition(
                                        |&c| !c.is_ascii_alphanumeric() && c != b'_'
                                    ).map_or(0, |q| q + 1);
                                    let word = &bytes[word_start..=p];
                                    is_regex = matches!(word,
                                        b"when" | b"if" | b"unless" | b"while" | b"until"
                                        | b"return" | b"and" | b"or" | b"not" | b"then"
                                        | b"else" | b"elsif" | b"do" | b"rescue" | b"in"
                                        | b"case" | b"yield");
                                }
                            }
                        }
                        if is_regex {
                            in_regex = true;
                            j += 1;
                            continue;
                        }
                    }
                    b'"' | b'\'' | b'`' => {
                        in_string = Some(b);
                    }
                    b'#' => break, // inline comment
                    b'?' => {
                        // A `?` starts a character literal when:
                        // 1. It is at the very start of the (trimmed) token — i.e., preceded by
                        //    whitespace or punctuation (or it is the first byte on the line), AND
                        // 2. The next byte is a printable ASCII character (not space, newline, or `?`)
                        let is_literal_start = if j == 0 {
                            true
                        } else {
                            is_char_literal_prefix(bytes[j - 1])
                        };

                        if is_literal_start && j + 1 < n {
                            let next = bytes[j + 1];
                            // next must be a printable ASCII char that is not space / `?`
                            if next.is_ascii_graphic() && next != b'?' {
                                let start = (line_start + j) as u32;
                                let end = start + 2; // `?` + one char
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message:
                                        "Do not use the character literal - use string literal instead."
                                            .into(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                                j += 2;
                                continue;
                            }
                        }
                    }
                    _ => {}
                }
                j += 1;
            }

            // If regex started but never closed on this line, it's multiline
            if in_regex {
                in_multiline_regex = true;
            }
        }

        diags
    }
}

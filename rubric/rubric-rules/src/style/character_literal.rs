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

impl Rule for CharacterLiteral {
    fn name(&self) -> &'static str {
        "Style/CharacterLiteral"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
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
                        let prev = bytes[..j].iter().rposition(|&c| c != b' ' && c != b'\t')
                            .map(|p| bytes[p]);
                        let is_regex = matches!(prev, None
                            | Some(b'(') | Some(b'[') | Some(b',') | Some(b'=')
                            | Some(b'!') | Some(b'|') | Some(b'&') | Some(b'?') | Some(b':')
                            | Some(b'{') | Some(b';') | Some(b'>') | Some(b'<'));
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
        }

        diags
    }
}

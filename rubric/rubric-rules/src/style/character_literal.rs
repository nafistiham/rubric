use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CharacterLiteral;

/// Returns true if this byte can precede a `?` that starts a character literal.
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
                while k < len && bytes[k] != q { k += 1; }
            } else {
                while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') { k += 1; }
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
        // Multiline percent literal state: (opener_byte, closer_byte, nesting_depth)
        // opener == closer for non-bracket delimiters (e.g. `%r|...|`)
        let mut in_multiline_pct: Option<(u8, u8, u32)> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // ── heredoc body ──────────────────────────────────────────────────
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // ── multiline percent literal body ────────────────────────────────
            if let Some((opener, closer, ref mut depth)) = in_multiline_pct {
                let bytes = line.as_bytes();
                let mut j = 0;
                while j < bytes.len() {
                    if bytes[j] == b'\\' { j += 2; continue; }
                    if opener != closer && bytes[j] == opener {
                        *depth += 1;
                    } else if bytes[j] == closer {
                        if *depth > 0 {
                            *depth -= 1;
                        } else {
                            in_multiline_pct = None;
                            break;
                        }
                    }
                    j += 1;
                }
                continue;
            }

            // ── multiline /regex/ body ────────────────────────────────────────
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

            // Detect heredoc opener (body starts on the NEXT line).
            let bytes = line.as_bytes();
            if let Some(term) = extract_heredoc_terminator(bytes) {
                in_heredoc = Some(term);
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            // Per-line percent literal: (opener, closer, depth)
            let mut pct: Option<(u8, u8, u32)> = None;
            let mut in_regex = false;
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                // ── inside /regex/ ────────────────────────────────────────────
                if in_regex {
                    match b {
                        b'\\' => { j += 2; continue; }
                        b'/' => { in_regex = false; }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                // ── inside percent literal ────────────────────────────────────
                if let Some((opener, closer, ref mut depth)) = pct {
                    if b == b'\\' { j += 2; continue; }
                    if opener != closer && b == opener {
                        *depth += 1;
                    } else if b == closer {
                        if *depth > 0 {
                            *depth -= 1;
                        } else {
                            pct = None;
                        }
                    }
                    j += 1;
                    continue;
                }

                // ── inside string ─────────────────────────────────────────────
                if let Some(delim) = in_string {
                    match b {
                        b'\\' => { j += 2; continue; }
                        c if c == delim => { in_string = None; }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                // ── normal scanning ───────────────────────────────────────────
                match b {
                    b'%' if j + 2 < n => {
                        let next = bytes[j + 1];
                        if matches!(next, b'r' | b'w' | b'W' | b'i' | b'I' | b'q' | b'Q' | b'x' | b's') {
                            let open = bytes[j + 2];
                            let (opener, closer) = match open {
                                b'(' => (b'(', b')'),
                                b'[' => (b'[', b']'),
                                b'{' => (b'{', b'}'),
                                b'<' => (b'<', b'>'),
                                c    => (c, c),
                            };
                            pct = Some((opener, closer, 0));
                            j += 3;
                            continue;
                        }
                    }
                    b'/' => {
                        let prev_pos = bytes[..j].iter().rposition(|&c| c != b' ' && c != b'\t');
                        let prev = prev_pos.map(|p| bytes[p]);
                        // `~` covers `=~` / `!~` operators
                        let mut is_regex = matches!(prev, None
                            | Some(b'(') | Some(b'[') | Some(b',') | Some(b'=')
                            | Some(b'!') | Some(b'|') | Some(b'&') | Some(b'?') | Some(b':')
                            | Some(b'{') | Some(b';') | Some(b'>') | Some(b'<') | Some(b'~'));
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
                        let is_literal_start = if j == 0 {
                            true
                        } else {
                            is_char_literal_prefix(bytes[j - 1])
                        };
                        if is_literal_start && j + 1 < n {
                            let next = bytes[j + 1];
                            if next.is_ascii_graphic() && next != b'?' {
                                let start = (line_start + j) as u32;
                                let end = start + 2;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Do not use the character literal - use string literal instead.".into(),
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

            // If a percent literal opened but didn't close, it's multiline.
            if let Some((opener, closer, depth)) = pct {
                in_multiline_pct = Some((opener, closer, depth));
            }
            // If a regex opened but didn't close, it's multiline.
            if in_regex {
                in_multiline_regex = true;
            }
        }

        diags
    }
}

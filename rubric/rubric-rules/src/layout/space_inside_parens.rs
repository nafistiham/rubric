use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceInsideParens;

/// If `line` opens a heredoc, return the terminator string.
fn extract_heredoc_terminator(line: &[u8]) -> Option<Vec<u8>> {
    let n = line.len();
    let mut i = 0;
    while i + 1 < n {
        if line[i] == b'<' && line[i + 1] == b'<' {
            i += 2;
            if i < n && (line[i] == b'-' || line[i] == b'~') { i += 1; }
            if i < n && (line[i] == b'\'' || line[i] == b'"' || line[i] == b'`') { i += 1; }
            let start = i;
            while i < n && (line[i].is_ascii_alphanumeric() || line[i] == b'_') { i += 1; }
            if i > start { return Some(line[start..i].to_vec()); }
        } else { i += 1; }
    }
    None
}

impl Rule for SpaceInsideParens {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideParens"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let mut in_heredoc: Option<Vec<u8>> = None;
        let mut in_percent_regex: bool = false;
        let mut percent_regex_depth: usize = 0;
        let mut in_percent_regex_close: u8 = b'}';

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();

            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                continue;
            }

            // Continue scanning through a multiline %r{...} body
            if in_percent_regex {
                let mut k = 0;
                while k < len {
                    let b = bytes[k];
                    if b == b'\\' { k += 2; continue; }
                    let open_bracket = match in_percent_regex_close {
                        b')' => b'(',
                        b']' => b'[',
                        b'}' => b'{',
                        b'>' => b'<',
                        c => c,
                    };
                    if b == in_percent_regex_close {
                        if percent_regex_depth == 0 {
                            in_percent_regex = false;
                            break;
                        }
                        percent_regex_depth -= 1;
                    } else if b == open_bracket {
                        percent_regex_depth += 1;
                    }
                    k += 1;
                }
                continue;
            }

            let mut in_string: Option<u8> = None;
            let mut in_regex = false;

            let mut j = 0;
            while j < len {
                let b = bytes[j];

                // Skip regex body — parens inside regex literals are not Ruby parens.
                if in_regex {
                    if b == b'\\' { j += 2; continue; }
                    if b == b'/' { in_regex = false; }
                    j += 1;
                    continue;
                }

                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // comment
                    None => {}
                }

                // Detect %r{...} (or %r(...) etc.) percent regex opener
                if b == b'%' && j + 1 < len && bytes[j + 1] == b'r' && j + 2 < len {
                    let delim = bytes[j + 2];
                    let close = match delim {
                        b'{' => Some(b'}'),
                        b'(' => Some(b')'),
                        b'[' => Some(b']'),
                        b'<' => Some(b'>'),
                        _ => None,
                    };
                    if let Some(close_byte) = close {
                        in_percent_regex = true;
                        in_percent_regex_close = close_byte;
                        percent_regex_depth = 0;
                        j += 3; // skip %r{
                        // Scan the rest of this line for the closing delimiter
                        while j < len {
                            let pb = bytes[j];
                            if pb == b'\\' { j += 2; continue; }
                            let open_bracket = match in_percent_regex_close {
                                b')' => b'(',
                                b']' => b'[',
                                b'}' => b'{',
                                b'>' => b'<',
                                c => c,
                            };
                            if pb == in_percent_regex_close {
                                if percent_regex_depth == 0 {
                                    in_percent_regex = false;
                                    j += 1;
                                    break;
                                }
                                percent_regex_depth -= 1;
                            } else if pb == open_bracket {
                                percent_regex_depth += 1;
                            }
                            j += 1;
                        }
                        continue;
                    }
                }

                // Detect regex opener: `/` preceded by `=`, `(`, `,`, `[`, space, tab, or
                // start-of-content. This avoids treating division `/` as a regex opener.
                if b == b'/' && in_string.is_none() {
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    if matches!(prev, b'=' | b'(' | b',' | b'[' | b' ' | b'\t' | 0) {
                        in_regex = true;
                        j += 1;
                        continue;
                    }
                }

                // Detect `( ` — open paren followed by space (skip `()`)
                if b == b'(' {
                    if j + 1 < len && bytes[j+1] == b' ' {
                        // Not an empty paren check needed since `( )` would also be caught by ` )`
                        let pos = (line_start + j + 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space after `(` detected.".into(),
                            range: TextRange::new(pos, pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Detect ` )` — space before close paren
                if b == b')' {
                    if j > 0 && bytes[j-1] == b' ' {
                        // Skip if `)` is the first non-space character (indented multiline close)
                        if bytes[..j].iter().any(|&b| b != b' ') {
                            let pos = (line_start + j - 1) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space before `)` detected.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }

                j += 1;
            }

            // Detect if this line opens a heredoc (body starts on the next line)
            if in_heredoc.is_none() {
                if let Some(term) = extract_heredoc_terminator(bytes) {
                    in_heredoc = Some(term);
                }
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: String::new(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

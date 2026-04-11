use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceInsideArrayLiteralBrackets;

impl Rule for SpaceInsideArrayLiteralBrackets {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideArrayLiteralBrackets"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut in_regex = false;

            let mut j = 0;
            while j < len {
                let b = bytes[j];

                // Skip regex body — brackets inside regex are character classes, not array brackets.
                if in_regex {
                    if b == b'\\' { j += 2; continue; }
                    if b == b'[' {
                        // Character class — skip until closing ] (handling escapes)
                        j += 1;
                        while j < len {
                            if bytes[j] == b'\\' { j += 2; continue; }
                            if bytes[j] == b']' { j += 1; break; }
                            j += 1;
                        }
                        continue;
                    }
                    if b == b'/' { in_regex = false; j += 1; continue; }
                    j += 1;
                    continue;
                }

                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break,
                    None if b == b'/' => {
                        // Detect regex start: preceding non-space char is NOT an identifier end
                        let prev = {
                            let mut k = j;
                            loop {
                                if k == 0 { break 0u8; }
                                k -= 1;
                                let pb = bytes[k];
                                if pb != b' ' && pb != b'\t' { break pb; }
                            }
                        };
                        if !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b')' || prev == b']') {
                            in_regex = true;
                        }
                        j += 1;
                        continue;
                    }
                    None => {}
                }

                // Skip non-array percent literals (%Q, %q, %r, %x, %s) — their [ ] are NOT array brackets
                if b == b'%' && j + 1 < len {
                    let (type_byte, delim_idx) = if bytes[j + 1].is_ascii_alphabetic() && j + 2 < len {
                        (bytes[j + 1], j + 2)
                    } else {
                        (b'Q', j + 1) // bare % acts like %Q
                    };
                    let delim = if delim_idx < len { bytes[delim_idx] } else { 0 };
                    let is_str_literal = matches!(type_byte, b'Q' | b'q' | b'r' | b'x' | b's');
                    // Also skip %w/%i with non-[ delimiters ([ delimiter handled below)
                    let is_array_nonbracket = matches!(type_byte, b'w' | b'W' | b'i' | b'I') && delim != b'[';
                    if (is_str_literal || is_array_nonbracket) && delim != 0 {
                        let close_delim = match delim {
                            b'(' => b')',
                            b'{' => b'}',
                            b'[' => b']',
                            b'<' => b'>',
                            other => other,
                        };
                        let paired = matches!(delim, b'(' | b'{' | b'[' | b'<');
                        j = delim_idx + 1;
                        let mut depth = 1usize;
                        while j < len {
                            let c = bytes[j];
                            if c == b'\\' { j += 2; continue; }
                            if paired {
                                if c == delim { depth += 1; }
                                else if c == close_delim { depth -= 1; if depth == 0 { j += 1; break; } }
                            } else if c == close_delim {
                                j += 1; break;
                            }
                            j += 1;
                        }
                        continue;
                    }
                }

                // Skip %w[...], %W[...], %i[...], %I[...] — [ is the delimiter, not an array bracket
                if b == b'%' && j + 2 < len && matches!(bytes[j + 1], b'w' | b'W' | b'i' | b'I') && bytes[j + 2] == b'[' {
                    j += 3; // skip %, letter, [
                    let mut depth = 1usize;
                    while j < len && depth > 0 {
                        match bytes[j] {
                            b'\\' => { j += 2; }
                            b'[' => { depth += 1; j += 1; }
                            b']' => { depth -= 1; j += 1; }
                            _ => { j += 1; }
                        }
                    }
                    continue;
                }

                // Detect `[ ` — open bracket followed by space (skip `[]` empty)
                if b == b'[' {
                    if j + 1 < len && bytes[j+1] == b' ' {
                        // Check it's not `[ ]` (empty array with space) — still flag it
                        let pos = (line_start + j + 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space after `[` detected.".into(),
                            range: TextRange::new(pos, pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Detect ` ]` — space before close bracket
                if b == b']' {
                    if j > 0 && bytes[j-1] == b' ' {
                        // Skip if `]` is the first non-space character (indented multiline close)
                        if bytes[..j].iter().any(|&b| b != b' ') {
                            let pos = (line_start + j - 1) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space before `]` detected.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }

                j += 1;
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

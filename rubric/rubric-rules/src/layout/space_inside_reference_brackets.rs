use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceInsideReferenceBrackets;

impl Rule for SpaceInsideReferenceBrackets {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideReferenceBrackets"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let n = bytes.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let mut pos = 0;
            let mut in_string: Option<u8> = None;
            let mut in_regex = false;

            while pos < n {
                let b = bytes[pos];

                // Skip regex body — brackets inside regex are character classes, not reference brackets.
                if in_regex {
                    if b == b'\\' { pos += 2; continue; }
                    if b == b'[' {
                        // Character class — skip until closing ] (handling escapes)
                        pos += 1;
                        while pos < n {
                            if bytes[pos] == b'\\' { pos += 2; continue; }
                            if bytes[pos] == b']' { pos += 1; break; }
                            pos += 1;
                        }
                        continue;
                    }
                    if b == b'/' { in_regex = false; pos += 1; continue; }
                    pos += 1;
                    continue;
                }

                match in_string {
                    Some(_) if b == b'\\' => { pos += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; pos += 1; continue; }
                    Some(_) => { pos += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); pos += 1; continue; }
                    None if b == b'#' => break,
                    None if b == b'/' => {
                        // Detect regex start: preceding non-space char is NOT an identifier end
                        let prev = {
                            let mut k = pos;
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
                        pos += 1;
                        continue;
                    }
                    None => {}
                }

                // Look for `[` followed by space (reference bracket, not array literal)
                // Simple heuristic: `[` after a word character
                if b == b'[' {
                    let prev = if pos > 0 { bytes[pos - 1] } else { 0 };
                    let next = if pos + 1 < n { bytes[pos + 1] } else { 0 };
                    // After a word char (indexing, not array literal)
                    if (prev.is_ascii_alphanumeric() || prev == b'_' || prev == b')' || prev == b']')
                        && next == b' '
                    {
                        let flag_pos = (line_start + pos + 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space detected inside reference brackets.".into(),
                            range: TextRange::new(flag_pos, flag_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Look for `]` preceded by space in reference context
                if b == b']' && pos > 0 {
                    let prev = bytes[pos - 1];
                    if prev == b' ' {
                        // Skip if `]` is the first non-space character (indented multiline close)
                        if bytes[..pos].iter().any(|&b| b != b' ') {
                            let flag_pos = (line_start + pos - 1) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space detected inside reference brackets.".into(),
                                range: TextRange::new(flag_pos, flag_pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }

                pos += 1;
            }
        }

        diags
    }
}

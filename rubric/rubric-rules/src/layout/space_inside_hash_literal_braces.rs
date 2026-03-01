use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceInsideHashLiteralBraces;

impl Rule for SpaceInsideHashLiteralBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideHashLiteralBraces"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;

            let mut j = 0;
            while j < len {
                let b = bytes[j];

                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break,
                    None => {}
                }

                // Detect `{` not followed by space and not empty `{}`
                if b == b'{' {
                    let next = if j + 1 < len { bytes[j+1] } else { 0 };
                    // Skip empty braces `{}`
                    if next == b'}' {
                        j += 2;
                        continue;
                    }
                    // Flag if next char is not a space
                    if next != b' ' && next != b'\n' && next != 0 {
                        let pos = (line_start + j + 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Missing space after `{` in hash literal.".into(),
                            range: TextRange::new(pos, pos),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Detect `}` not preceded by space and not empty `{}`
                if b == b'}' {
                    let prev = if j > 0 { bytes[j-1] } else { 0 };
                    // Skip empty braces already handled above
                    if prev == b'{' {
                        j += 1;
                        continue;
                    }
                    // Flag if prev char is not a space
                    if prev != b' ' && prev != 0 {
                        let pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Missing space before `}` in hash literal.".into(),
                            range: TextRange::new(pos, pos),
                            severity: Severity::Warning,
                        });
                    }
                }

                j += 1;
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // Insert a space at the flagged position
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: " ".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

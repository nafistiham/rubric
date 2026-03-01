use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceInsideParens;

impl Rule for SpaceInsideParens {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideParens"
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
                    None if b == b'#' => break, // comment
                    None => {}
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
                        let pos = (line_start + j - 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space before `)` detected.".into(),
                            range: TextRange::new(pos, pos + 1),
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
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: String::new(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

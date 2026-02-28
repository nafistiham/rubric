use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceAfterComma;

impl Rule for SpaceAfterComma {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterComma"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let line_bytes = line.as_bytes();
            for (j, &b) in line_bytes.iter().enumerate() {
                if b == b',' {
                    let next = line_bytes.get(j + 1).copied();
                    // flag if next char is not space and not end of line
                    if next != Some(b' ') && next != Some(b'\t') && next.is_some() {
                        let pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space missing after comma.".into(),
                            range: TextRange::new(pos, pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: TextRange::new(diag.range.start, diag.range.end),
                replacement: ", ".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

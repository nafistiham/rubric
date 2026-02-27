use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct TrailingWhitespace;

impl Rule for TrailingWhitespace {
    fn name(&self) -> &'static str {
        "Layout/TrailingWhitespace"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed_len = line.trim_end().len();
            let trailing = line.len() - trimmed_len;
            if trailing == 0 {
                continue;
            }
            let line_start = ctx.line_start_offsets[i];
            let start = line_start + trimmed_len as u32;
            let end   = line_start + line.len() as u32;
            diagnostics.push(Diagnostic {
                rule: self.name(),
                message: "Trailing whitespace detected.".into(),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diagnostics
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

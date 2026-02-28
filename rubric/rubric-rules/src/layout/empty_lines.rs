use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLines;

impl Rule for EmptyLines {
    fn name(&self) -> &'static str {
        "Layout/EmptyLines"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut blank_run = 0usize;
        for (i, line) in ctx.lines.iter().enumerate() {
            if line.trim().is_empty() {
                blank_run += 1;
                if blank_run == 2 {
                    let start = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra blank line detected.".into(),
                        range: TextRange::new(start, start),
                        severity: Severity::Warning,
                    });
                }
            } else {
                blank_run = 0;
            }
        }
        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct LineLength;

const MAX: usize = 120;

impl Rule for LineLength {
    fn name(&self) -> &'static str {
        "Layout/LineLength"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            if line.len() > MAX {
                let start = ctx.line_start_offsets[i];
                let end = start + line.len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!("Line is {} characters (max is {MAX}).", line.len()),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }
        diags
    }
}

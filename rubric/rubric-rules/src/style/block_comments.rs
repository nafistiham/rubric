use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct BlockComments;

impl Rule for BlockComments {
    fn name(&self) -> &'static str {
        "Style/BlockComments"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            if line.starts_with("=begin") {
                let start = ctx.line_start_offsets[i];
                let end = start + line.len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Do not use block comments.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

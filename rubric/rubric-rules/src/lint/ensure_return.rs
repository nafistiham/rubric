use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EnsureReturn;

impl Rule for EnsureReturn {
    fn name(&self) -> &'static str {
        "Lint/EnsureReturn"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut in_ensure = false;

        for i in 0..n {
            let trimmed = lines[i].trim();

            if trimmed == "ensure" {
                in_ensure = true;
                continue;
            }

            if trimmed == "end" {
                in_ensure = false;
                continue;
            }

            if in_ensure && (trimmed.starts_with("return ") || trimmed == "return") {
                let indent = lines[i].len() - lines[i].trim_start().len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Do not use `return` in an `ensure` block.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

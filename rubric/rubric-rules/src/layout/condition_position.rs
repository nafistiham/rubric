use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ConditionPosition;

impl Rule for ConditionPosition {
    fn name(&self) -> &'static str {
        "Layout/ConditionPosition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `if ... then ...` or `unless ... then ...` on one line
            let has_inline_then = (trimmed.starts_with("if ") || trimmed.starts_with("unless "))
                && trimmed.contains(" then ");

            if has_inline_then {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Avoid using `then` with `if`/`unless`.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

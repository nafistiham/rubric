use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NegatedIf;

impl Rule for NegatedIf {
    fn name(&self) -> &'static str {
        "Style/NegatedIf"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Match `if !` at the start of a line (allowing indentation)
            if !trimmed.starts_with("if ") {
                continue;
            }

            let after_if = trimmed["if ".len()..].trim_start();
            if !after_if.starts_with('!') {
                continue;
            }

            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let pos = (line_start + indent) as u32;

            diags.push(Diagnostic {
                rule: self.name(),
                message: "Use `unless` instead of `if !`.".into(),
                range: TextRange::new(pos, pos + 2),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

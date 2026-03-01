use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AccessModifierDeclarations;

impl Rule for AccessModifierDeclarations {
    fn name(&self) -> &'static str {
        "Style/AccessModifierDeclarations"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let has_inline = trimmed.starts_with("private def ")
                || trimmed.starts_with("protected def ")
                || trimmed.starts_with("public def ");

            if has_inline {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Access modifiers should be on their own line, not inline with `def`.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

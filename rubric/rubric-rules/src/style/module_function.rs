use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ModuleFunction;

impl Rule for ModuleFunction {
    fn name(&self) -> &'static str {
        "Style/ModuleFunction"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `module_function` on its own line (without arguments)
            if trimmed == "module_function" {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Prefer `module_function :method_name` style over bare `module_function`.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

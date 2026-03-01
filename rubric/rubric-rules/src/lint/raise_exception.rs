use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RaiseException;

impl Rule for RaiseException {
    fn name(&self) -> &'static str {
        "Lint/RaiseException"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `raise Exception` (not `raise SomeOtherException`)
            if trimmed.starts_with("raise Exception") {
                let after = &trimmed["raise Exception".len()..];
                let is_exception_class = after.is_empty()
                    || after.starts_with(' ')
                    || after.starts_with('.')
                    || after.starts_with(',');

                if is_exception_class {
                    let indent = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `StandardError` instead of `Exception`; catching `Exception` is dangerous.".into(),
                        range: TextRange::new(pos, pos + "raise Exception".len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

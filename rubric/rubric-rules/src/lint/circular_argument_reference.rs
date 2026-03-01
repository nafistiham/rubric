use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CircularArgumentReference;

impl Rule for CircularArgumentReference {
    fn name(&self) -> &'static str {
        "Lint/CircularArgumentReference"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("def ") {
                continue;
            }

            // Find the parameter list
            let paren_start = match trimmed.find('(') {
                Some(p) => p,
                None => continue,
            };
            let paren_end = match trimmed.rfind(')') {
                Some(p) => p,
                None => continue,
            };

            if paren_end <= paren_start {
                continue;
            }

            let params_str = &trimmed[paren_start + 1..paren_end];

            // Split by comma and look for `param = param` pattern
            for param_def in params_str.split(',') {
                let p = param_def.trim();
                // Look for `name = name` where same name appears on both sides
                if let Some(eq_pos) = p.find(" = ") {
                    let param_name = p[..eq_pos].trim();
                    let default_val = p[eq_pos + 3..].trim();
                    if param_name == default_val && !param_name.is_empty() {
                        let indent = line.len() - trimmed.len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "Circular argument reference: `{}` references itself in its default.",
                                param_name
                            ),
                            range: TextRange::new(pos, pos + trimmed.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }
}

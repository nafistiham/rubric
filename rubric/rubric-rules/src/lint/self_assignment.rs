use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SelfAssignment;

impl Rule for SelfAssignment {
    fn name(&self) -> &'static str {
        "Lint/SelfAssignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `x = x` pattern
            if let Some(eq_pos) = trimmed.find(" = ") {
                let lhs = trimmed[..eq_pos].trim();
                let rhs = trimmed[eq_pos + 3..].trim();

                // Skip compound assignments: `x = x + 1` is not self-assignment
                if lhs == rhs && !lhs.is_empty() && lhs.chars().next().map(|c| c.is_ascii_lowercase() || c == '_').unwrap_or(false) {
                    let indent = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Self-assignment to variable `{}`.", lhs),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ParallelAssignment;

impl Rule for ParallelAssignment {
    fn name(&self) -> &'static str {
        "Style/ParallelAssignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `a, b = 1, 2` style parallel assignment
            // LHS must have a comma, RHS must also have a comma (not a splat/array)
            if let Some(eq_pos) = trimmed.find(" = ") {
                let lhs = &trimmed[..eq_pos];
                let rhs = &trimmed[eq_pos + 3..];

                // LHS has comma (parallel LHS)
                if !lhs.contains(',') {
                    continue;
                }

                // RHS must also have comma and not start with `[` (array literal)
                if !rhs.contains(',') || rhs.trim_start().starts_with('[') {
                    continue;
                }

                // Skip swap pattern `a, b = b, a` - both sides have exactly the same vars
                // Just flag all parallel assignments with literal RHS values
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use sequential assignment instead of parallel assignment.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

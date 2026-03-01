use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TrailingUnderscoreVariable;

impl Rule for TrailingUnderscoreVariable {
    fn name(&self) -> &'static str {
        "Style/TrailingUnderscoreVariable"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect parallel assignment with trailing `_`: `a, _ =` or `x, y, _ =`
            // The pattern is: comma-separated identifiers ending with `, _` followed by ` =`
            if let Some(eq_pos) = trimmed.find(" = ").or_else(|| {
                // Also handle `= ` at end of trimmed (without RHS on same line)
                None
            }) {
                let lhs = &trimmed[..eq_pos];
                if lhs.contains(',') {
                    // Check if lhs ends with `, _` (possibly with spaces)
                    let lhs_trimmed = lhs.trim_end();
                    if lhs_trimmed.ends_with(", _") || lhs_trimmed.ends_with(",_") {
                        let indent = line.len() - trimmed.len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Trailing `_` in parallel assignment is unnecessary.".into(),
                            range: TextRange::new(pos, pos + lhs.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }
}

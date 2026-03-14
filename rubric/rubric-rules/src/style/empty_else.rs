use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyElse;

impl Rule for EmptyElse {
    fn name(&self) -> &'static str {
        "Style/EmptyElse"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        const MESSAGE: &str = "Avoid empty else-clauses.";

        let lines = &ctx.lines;
        let offsets = &ctx.line_start_offsets;

        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim();

            // Skip comment lines
            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Look for a line that is exactly `else` (with optional indentation)
            if trimmed == "else" {
                // Check the next non-empty line
                let next = i + 1;
                if next < lines.len() {
                    let next_trimmed = lines[next].trim();

                    // Empty else: `else` immediately followed by `end`
                    // Nil else: `else` followed by `nil`
                    if next_trimmed == "end" || next_trimmed == "nil" {
                        let line_start = offsets[i] as usize;
                        // Flag from the start of `else` keyword in the line
                        let indent = lines[i].len() - lines[i].trim_start().len();
                        let start = (line_start + indent) as u32;
                        let end = start + "else".len() as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: MESSAGE.into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            i += 1;
        }

        diags
    }
}

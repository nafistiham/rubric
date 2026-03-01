use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TrailingCommaInArrayLiteral;

impl Rule for TrailingCommaInArrayLiteral {
    fn name(&self) -> &'static str {
        "Style/TrailingCommaInArrayLiteral"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Look for a line ending with `,` (trailing comma) where the NEXT
        // non-blank line is `]` (closing bracket on its own line).
        for i in 0..n {
            let trimmed = lines[i].trim_end();
            if trimmed.ends_with(',') {
                // Check if the next line is `]`
                if i + 1 < n {
                    let next = lines[i + 1].trim();
                    if next == "]" || next.starts_with("] ") || next.starts_with("],") {
                        // Trailing comma before closing bracket
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let comma_pos = trimmed.len() - 1;
                        let abs_pos = (line_start + comma_pos) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Trailing comma in array literal.".into(),
                            range: TextRange::new(abs_pos, abs_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultipleComparison;

impl Rule for MultipleComparison {
    fn name(&self) -> &'static str {
        "Lint/MultipleComparison"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect chained comparisons like `1 < x < 10` or `a <= b <= c`
            // Pattern: something <op> something <op> something
            let ops = ["<=", ">=", "<", ">"];
            for &op1 in &ops {
                if let Some(pos1) = line.find(op1) {
                    let rest = &line[pos1 + op1.len()..];
                    // Skip whitespace
                    let rest_trimmed = rest.trim_start();
                    // Find the next comparison in what follows
                    for &op2 in &ops {
                        // Look for another comparison operator after some content
                        if let Some(pos2) = rest_trimmed.find(op2) {
                            if pos2 > 0 {
                                // Make sure there's an identifier/value between them
                                let between = &rest_trimmed[..pos2].trim();
                                if !between.is_empty() && !between.contains('&') && !between.contains('|') {
                                    let indent = line.len() - trimmed.len();
                                    let line_start = ctx.line_start_offsets[i] as usize;
                                    let pos = (line_start + indent) as u32;
                                    diags.push(Diagnostic {
                                        rule: self.name(),
                                        message: "Chained comparison does not work as expected in Ruby.".into(),
                                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                                        severity: Severity::Warning,
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        diags
    }
}

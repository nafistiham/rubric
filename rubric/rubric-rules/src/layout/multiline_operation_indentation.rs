use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineOperationIndentation;

// Operators that can appear at end of line for continuation
const CONTINUATION_OPS: &[&str] = &["||", "&&", "**", "+=", "-=", "*=", "/=", "+", "-", "*", "/"];

fn ends_with_operator(line: &str) -> bool {
    let trimmed = line.trim_end();
    for op in CONTINUATION_OPS {
        if trimmed.ends_with(op) {
            // Make sure it's not a comment-looking thing
            // Check that op is preceded by a space or alphanumeric
            let without_op = &trimmed[..trimmed.len() - op.len()];
            if without_op.is_empty() || without_op.ends_with(' ') || without_op.ends_with(|c: char| c.is_ascii_alphanumeric()) {
                return true;
            }
        }
    }
    false
}

impl Rule for MultilineOperationIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineOperationIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            if ends_with_operator(line) && i + 1 < n {
                let current_indent = line.len() - trimmed.len();
                let next_line = &lines[i + 1];
                let next_trimmed = next_line.trim_start();

                // Skip blank lines
                if next_trimmed.is_empty() {
                    i += 1;
                    continue;
                }

                let next_indent = next_line.len() - next_trimmed.len();
                let expected_indent = current_indent + 2;

                if next_indent != expected_indent {
                    let line_start = ctx.line_start_offsets[i + 1];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Continuation line indentation ({}) should be {} (current + 2).",
                            next_indent, expected_indent
                        ),
                        range: TextRange::new(line_start, line_start + next_indent as u32),
                        severity: Severity::Warning,
                    });
                }
            }
            i += 1;
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ElseAlignment;

impl Rule for ElseAlignment {
    fn name(&self) -> &'static str {
        "Layout/ElseAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Stack of (keyword, indent) for if/unless/while/until
        let mut if_stack: Vec<usize> = Vec::new();

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("if ") || trimmed.starts_with("unless ") {
                if_stack.push(indent);
            } else if trimmed == "else" || trimmed.starts_with("elsif ") {
                if let Some(&expected_indent) = if_stack.last() {
                    if indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`else`/`elsif` at indent {} should be at {}.",
                                indent, expected_indent
                            ),
                            range: TextRange::new(pos, pos + trimmed.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }
            } else if trimmed == "end" {
                if_stack.pop();
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EndAlignment;

impl Rule for EndAlignment {
    fn name(&self) -> &'static str {
        "Layout/EndAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Stack of (keyword, indentation) for openers
        let mut indent_stack: Vec<usize> = Vec::new();

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with("def ") || trimmed == "def"
                || trimmed.starts_with("class ") || trimmed.starts_with("module ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed.starts_with("begin")
                || trimmed == "do" || trimmed.ends_with(" do") {
                indent_stack.push(indent);
            } else if trimmed == "end" || trimmed.starts_with("end ") {
                if let Some(expected_indent) = indent_stack.pop() {
                    if indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`end` indentation ({}) does not match its opener ({}).",
                                indent, expected_indent
                            ),
                            range: TextRange::new(
                                line_start + indent as u32,
                                line_start + indent as u32 + 3,
                            ),
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

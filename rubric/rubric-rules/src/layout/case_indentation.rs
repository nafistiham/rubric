use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CaseIndentation;

impl Rule for CaseIndentation {
    fn name(&self) -> &'static str {
        "Layout/CaseIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track current `case` indentation
        // Stack of case indentations
        let mut case_stack: Vec<usize> = Vec::new();
        // General depth tracking for other openers
        let mut depth_stack: Vec<bool> = Vec::new(); // true = case opener

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with("case ") || trimmed == "case" {
                case_stack.push(indent);
                depth_stack.push(true);
            } else if trimmed.starts_with("def ") || trimmed == "def"
                || trimmed.starts_with("class ") || trimmed.starts_with("module ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed.starts_with("begin")
                || trimmed == "do" || trimmed.ends_with(" do") {
                depth_stack.push(false);
            } else if trimmed == "end" || trimmed.starts_with("end ") {
                if let Some(is_case) = depth_stack.pop() {
                    if is_case {
                        case_stack.pop();
                    }
                }
            } else if trimmed.starts_with("when ") || trimmed == "when" {
                // `when` should be at the same indentation as `case`
                if let Some(&case_indent) = case_stack.last() {
                    if indent != case_indent {
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`when` indentation ({}) does not match `case` indentation ({}).",
                                indent, case_indent
                            ),
                            range: TextRange::new(
                                line_start + indent as u32,
                                line_start + indent as u32 + 4,
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

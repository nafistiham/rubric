use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RescueEnsureAlignment;

impl Rule for RescueEnsureAlignment {
    fn name(&self) -> &'static str {
        "Layout/RescueEnsureAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track the indentation of begin/def openers on a stack
        // Each entry: indentation level of the opener
        let mut indent_stack: Vec<usize> = Vec::new();

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            // Detect `do` block openers: `foo do`, `foo do |x|`, etc.
            let opens_do_block = (trimmed.ends_with(" do")
                || trimmed.contains(" do |")
                || trimmed.contains(" do\n"))
                && !trimmed.starts_with("def ")
                && trimmed != "def";

            if trimmed.starts_with("begin") || trimmed.starts_with("def ")
                || trimmed == "def" || opens_do_block {
                indent_stack.push(indent);
            } else if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end.") {
                indent_stack.pop();
            } else if trimmed.starts_with("rescue") || trimmed.starts_with("ensure") {
                // Check alignment: should match the most recent begin/def indent
                if let Some(&expected_indent) = indent_stack.last() {
                    if indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i];
                        let keyword_end = if trimmed.starts_with("rescue") {
                            line_start + indent as u32 + "rescue".len() as u32
                        } else {
                            line_start + indent as u32 + "ensure".len() as u32
                        };
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`rescue`/`ensure` indentation ({}) does not match its opener ({}).",
                                indent, expected_indent
                            ),
                            range: TextRange::new(line_start + indent as u32, keyword_end),
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

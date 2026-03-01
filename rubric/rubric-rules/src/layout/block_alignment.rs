use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct BlockAlignment;

impl Rule for BlockAlignment {
    fn name(&self) -> &'static str {
        "Layout/BlockAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track `do` block openings with their indentation
        let mut block_stack: Vec<usize> = Vec::new(); // indent levels of `do` lines

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            let opens_block = trimmed.ends_with(" do")
                || trimmed.ends_with(" do |")
                || trimmed.contains(" do |")
                || (trimmed == "do");

            if opens_block {
                block_stack.push(indent);
            }

            if trimmed == "end" {
                if let Some(expected_indent) = block_stack.pop() {
                    if indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`end` at indent {} does not match block start at indent {}.",
                                indent, expected_indent
                            ),
                            range: TextRange::new(pos, pos + 3),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }
}

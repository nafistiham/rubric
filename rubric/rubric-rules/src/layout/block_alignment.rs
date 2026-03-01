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
        // Track inner construct depth so `end` for if/def/class/etc. doesn't pop block stack
        let mut inner_depth = 0usize;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            let t = trimmed.trim();

            let opens_block = t.ends_with(" do")
                || t.ends_with(" do |")
                || t.contains(" do |")
                || t.contains(" do|")
                || t == "do";

            let opens_inner = !opens_block && (
                t.starts_with("def ")
                    || t == "def"
                    || t.starts_with("if ")
                    || t.starts_with("unless ")
                    || t.starts_with("while ")
                    || t.starts_with("until ")
                    || t == "begin"
                    || t.starts_with("begin ")
                    || t.starts_with("case ")
                    || t.starts_with("class ")
                    || t.starts_with("module ")
            );

            if opens_block {
                block_stack.push(indent);
            } else if opens_inner && !block_stack.is_empty() {
                inner_depth += 1;
            }

            if t == "end" {
                if inner_depth > 0 {
                    inner_depth -= 1;
                } else if let Some(expected_indent) = block_stack.pop() {
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

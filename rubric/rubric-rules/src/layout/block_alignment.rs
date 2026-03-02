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

        // Unified stack: (indent, is_do_block)
        // true = do-block (alignment checked), false = inner construct (alignment not checked)
        let mut stack: Vec<(usize, bool)> = Vec::new();

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            let t = trimmed.trim();

            let opens_do_block = t.ends_with(" do")
                || t.ends_with(" do |")
                || t.contains(" do |")
                || t.contains(" do|")
                || t == "do";

            let opens_inner_construct = !opens_do_block && (
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

            let has_inline_conditional = !opens_do_block && !opens_inner_construct && (
                t.contains(" << if ") || t.contains(" << unless ") || t.contains(" << case ")
                || t.contains(" = if ") || t.contains(" = unless ") || t.contains(" = case ")
            );

            if opens_do_block {
                stack.push((indent, true));
            } else if opens_inner_construct || has_inline_conditional {
                // Only track if we're inside a do-block (stack not empty)
                if !stack.is_empty() {
                    stack.push((indent, false));
                }
            }

            if t == "end" || t.starts_with("end ") || t.starts_with("end.") {
                if let Some((expected_indent, is_do)) = stack.pop() {
                    if is_do && indent != expected_indent {
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
                    // If not is_do (inner construct), just pop silently — no alignment check
                }
            }
        }

        diags
    }
}

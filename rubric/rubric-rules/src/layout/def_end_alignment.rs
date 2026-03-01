use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DefEndAlignment;

impl Rule for DefEndAlignment {
    fn name(&self) -> &'static str {
        "Layout/DefEndAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track def lines with their indentation
        let mut def_stack: Vec<(usize, usize)> = Vec::new(); // (line_idx, indent)

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("def ") || trimmed == "def" {
                def_stack.push((i, indent));
            } else if trimmed == "end" {
                if let Some((_def_line, expected_indent)) = def_stack.pop() {
                    if indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`end` at indent {} does not match `def` at indent {}.",
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

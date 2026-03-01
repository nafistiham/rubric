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
        // Track inner construct depth (if/unless/while/until/begin/case/class/module)
        // so that an `end` for an inner construct doesn't pop the outer def.
        let mut inner_depth = 0usize;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("def ") || trimmed == "def" {
                def_stack.push((i, indent));
            } else if !def_stack.is_empty() {
                // Track inner constructs that consume `end` tokens
                let t = trimmed.trim();
                let opens_inner = t.starts_with("if ")
                    || t.starts_with("unless ")
                    || t.starts_with("while ")
                    || t.starts_with("until ")
                    || t == "begin"
                    || t.starts_with("begin ")
                    || t.starts_with("case ")
                    || t.starts_with("class ")
                    || t.starts_with("module ")
                    || t.ends_with(" do")
                    || t.contains(" do |")
                    || t.contains(" do|");

                if opens_inner {
                    inner_depth += 1;
                } else if t == "end" {
                    if inner_depth > 0 {
                        inner_depth -= 1;
                    } else if let Some((_def_line, expected_indent)) = def_stack.pop() {
                        inner_depth = 0;
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
        }

        diags
    }
}

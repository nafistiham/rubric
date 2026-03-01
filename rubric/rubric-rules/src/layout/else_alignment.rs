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

        // Stack of indent for if/unless
        let mut if_stack: Vec<usize> = Vec::new();
        // Track depth of non-if constructs that consume `end` tokens
        // so they don't pop the if stack prematurely
        let mut other_depth = 0usize;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            let t = trimmed.trim();

            if t.starts_with("if ") || t.starts_with("unless ") {
                if_stack.push(indent);
            } else if t.starts_with("def ")
                || t == "def"
                || t == "begin"
                || t.starts_with("begin ")
                || t.starts_with("while ")
                || t.starts_with("until ")
                || t.starts_with("case ")
                || t.starts_with("class ")
                || t.starts_with("module ")
                || t.ends_with(" do")
                || t.contains(" do |")
                || t.contains(" do|")
            {
                other_depth += 1;
            } else if t == "else" || t.starts_with("elsif ") {
                if other_depth == 0 {
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
                }
            } else if t == "end" {
                if other_depth > 0 {
                    other_depth -= 1;
                } else {
                    if_stack.pop();
                }
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UnlessElse;

impl Rule for UnlessElse {
    fn name(&self) -> &'static str {
        "Style/UnlessElse"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut in_unless = false;
        let mut depth = 0i32;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Detect start of an `unless` block
            if trimmed.starts_with("unless ") || trimmed == "unless" {
                in_unless = true;
                depth = 1;
                continue;
            }

            if in_unless {
                // Track nested block depth
                if trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                    || trimmed.starts_with("def ") || trimmed.starts_with("do ")
                    || trimmed.starts_with("begin") || trimmed.starts_with("case ")
                    || trimmed == "do"
                {
                    depth += 1;
                }

                if trimmed == "end" {
                    depth -= 1;
                    if depth <= 0 {
                        in_unless = false;
                    }
                }

                // Detect `else` at depth 1 (direct child of unless)
                if depth == 1 && trimmed == "else" {
                    let line_start = ctx.line_start_offsets[i] as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Avoid `unless` with `else` — use `if` instead.".into(),
                        range: TextRange::new(line_start, line_start + 4),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

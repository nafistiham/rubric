use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NoReturnInBeginEndBlock;

impl Rule for NoReturnInBeginEndBlock {
    fn name(&self) -> &'static str {
        "Lint/NoReturnInBeginEndBlock"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Detect `BEGIN { return ... }` or `END { return ... }` on same line
            if (trimmed.starts_with("BEGIN {") || trimmed.starts_with("END {"))
                && trimmed.contains("return")
            {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Do not use `return` in a `BEGIN`/`END` block.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

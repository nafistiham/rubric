use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EndBlock;

impl Rule for EndBlock {
    fn name(&self) -> &'static str {
        "Style/EndBlock"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Match `END` followed by optional whitespace and `{`
            if trimmed.starts_with("END") {
                let after_end = trimmed[3..].trim_start();
                if after_end.starts_with('{') {
                    let start = ctx.line_start_offsets[i];
                    let end = start + line.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Avoid the use of END blocks. Use Kernel#at_exit instead.".into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

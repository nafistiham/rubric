use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct StderrPuts;

impl Rule for StderrPuts {
    fn name(&self) -> &'static str {
        "Style/StderrPuts"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find("$stderr.puts") {
                let line_start = ctx.line_start_offsets[i] as usize;
                let abs_pos = (line_start + pos) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `warn` instead of `$stderr.puts`.".into(),
                    range: TextRange::new(abs_pos, abs_pos + "$stderr.puts".len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

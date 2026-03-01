use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct WhileUntilDo;

impl Rule for WhileUntilDo {
    fn name(&self) -> &'static str {
        "Style/WhileUntilDo"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Check for `while ... do` or `until ... do` at end of trimmed line
            let is_while = trimmed.starts_with("while ");
            let is_until = trimmed.starts_with("until ");

            if (is_while || is_until) && trimmed.trim_end().ends_with(" do") {
                let line_start = ctx.line_start_offsets[i] as usize;
                let trimmed_end = line.trim_end();
                let do_pos = trimmed_end.len() - 3; // " do" is 3 chars
                let abs_pos = (line_start + do_pos) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Redundant `do` after `while`/`until` condition.".into(),
                    range: TextRange::new(abs_pos, abs_pos + 3),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

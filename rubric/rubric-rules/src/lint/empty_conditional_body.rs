use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyConditionalBody;

impl Rule for EmptyConditionalBody {
    fn name(&self) -> &'static str {
        "Lint/EmptyConditionalBody"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 0..n {
            let trimmed = lines[i].trim();
            if !trimmed.starts_with("if ") && !trimmed.starts_with("unless ") {
                continue;
            }

            // Check if the next non-empty line is `end` or `else` (empty body)
            if i + 1 < n {
                let next = lines[i + 1].trim();
                if next == "end" || next == "else" {
                    let indent = lines[i].len() - lines[i].trim_start().len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Empty conditional body detected.".into(),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

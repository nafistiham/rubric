use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantCondition;

impl Rule for RedundantCondition {
    fn name(&self) -> &'static str {
        "Style/RedundantCondition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let trimmed = lines[i].trim();
            if !trimmed.starts_with("if ") && !trimmed.starts_with("unless ") {
                i += 1;
                continue;
            }

            // Look for: if <cond>\n  true\nelse\n  false\nend
            // or: if <cond>\n  false\nelse\n  true\nend
            if i + 4 >= n {
                i += 1;
                continue;
            }

            let then_val = lines[i + 1].trim();
            let else_kw = lines[i + 2].trim();
            let else_val = lines[i + 3].trim();
            let end_kw = lines[i + 4].trim();

            if else_kw != "else" || end_kw != "end" {
                i += 1;
                continue;
            }

            let is_redundant = (then_val == "true" && else_val == "false")
                || (then_val == "false" && else_val == "true");

            if is_redundant {
                let indent = lines[i].len() - lines[i].trim_start().len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Redundant condition; use the condition directly.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
                i += 5;
                continue;
            }

            i += 1;
        }

        diags
    }
}

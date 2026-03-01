use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DuplicateBranch;

impl Rule for DuplicateBranch {
    fn name(&self) -> &'static str {
        "Lint/DuplicateBranch"
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

            // Simple: look for `if\n  <body1>\nelse\n  <body2>\nend` where body1 == body2
            if i + 4 >= n {
                i += 1;
                continue;
            }

            let then_body = lines[i + 1].trim();
            let else_kw = lines[i + 2].trim();
            let else_body = lines[i + 3].trim();
            let end_kw = lines[i + 4].trim();

            if else_kw == "else" && end_kw == "end" && then_body == else_body && !then_body.is_empty() {
                let indent = lines[i].len() - lines[i].trim_start().len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Duplicate branch body detected in if/else.".into(),
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

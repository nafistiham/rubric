use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantBegin;

impl Rule for RedundantBegin {
    fn name(&self) -> &'static str {
        "Style/RedundantBegin"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Detect `def` followed immediately by `begin` as first non-blank line.
        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("def ") || trimmed == "def" {
                // Find the first non-blank line after `def`
                let mut j = i + 1;
                while j < n && lines[j].trim().is_empty() {
                    j += 1;
                }
                if j < n && lines[j].trim() == "begin" {
                    let line_start = ctx.line_start_offsets[j];
                    let indent = lines[j].len() - lines[j].trim_start().len();
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Redundant `begin` block in method body.".into(),
                        range: TextRange::new(
                            line_start + indent as u32,
                            line_start + indent as u32 + 5,
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
            i += 1;
        }

        diags
    }
}

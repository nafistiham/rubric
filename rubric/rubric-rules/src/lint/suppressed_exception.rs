use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SuppressedException;

impl Rule for SuppressedException {
    fn name(&self) -> &'static str {
        "Lint/SuppressedException"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Detect `rescue` line immediately followed by `end` with no code between
        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim_start();
            if trimmed == "rescue" || trimmed.starts_with("rescue ") || trimmed.starts_with("rescue\t") {
                // Check next non-blank line is `end`
                let mut j = i + 1;
                while j < n && lines[j].trim().is_empty() {
                    j += 1;
                }
                if j < n && lines[j].trim() == "end" {
                    let line_start = ctx.line_start_offsets[i];
                    let indent = lines[i].len() - trimmed.len();
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Empty `rescue` block suppresses exceptions.".into(),
                        range: TextRange::new(
                            line_start + indent as u32,
                            line_start + indent as u32 + 6,
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

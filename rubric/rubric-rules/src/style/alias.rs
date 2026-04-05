use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct Alias;

impl Rule for Alias {
    fn name(&self) -> &'static str {
        "Style/Alias"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comments
            if trimmed.starts_with('#') {
                continue;
            }

            // RuboCop's default EnforcedStyle is `prefer_alias`:
            // flag `alias_method` and recommend using `alias` keyword.
            if !trimmed.starts_with("alias_method") {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let indent = line.len() - trimmed.len();
            let start = (line_start + indent) as u32;
            let end = start + 12; // `alias_method`

            diags.push(Diagnostic {
                rule: self.name(),
                message: "Use `alias` instead of `alias_method`.".into(),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

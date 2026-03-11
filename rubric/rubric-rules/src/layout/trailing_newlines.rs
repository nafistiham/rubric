use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TrailingNewlines;

impl Rule for TrailingNewlines {
    fn name(&self) -> &'static str {
        "Layout/TrailingNewlines"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let source = ctx.source;
        // Empty files are excluded (rubocop does not flag them).
        if source.is_empty() {
            return vec![];
        }
        if source.ends_with("\n\n") || !source.ends_with('\n') {
            let offset = source.len().saturating_sub(1) as u32;
            let msg = if source.ends_with("\n\n") {
                "Extra blank lines at end of file."
            } else {
                "File must end with a newline."
            };
            vec![Diagnostic {
                rule: self.name(),
                message: msg.into(),
                range: TextRange::new(offset, offset + 1),
                severity: Severity::Warning,
            }]
        } else {
            vec![]
        }
    }
}

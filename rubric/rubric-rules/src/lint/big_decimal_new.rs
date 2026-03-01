use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct BigDecimalNew;

impl Rule for BigDecimalNew {
    fn name(&self) -> &'static str {
        "Lint/BigDecimalNew"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;
        let pattern = "BigDecimal.new(";

        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find(pattern) {
            let abs_pos = search_start + pos;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Use `BigDecimal(...)` instead of `BigDecimal.new(...)`.".into(),
                range: TextRange::new(abs_pos as u32, (abs_pos + pattern.len()) as u32),
                severity: Severity::Warning,
            });
            search_start = abs_pos + pattern.len();
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: "BigDecimal(".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

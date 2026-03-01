use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceInsideRangeLiteral;

impl Rule for SpaceInsideRangeLiteral {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideRangeLiteral"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        // Detect ` .. ` or ` ... ` (space before and after range operator)
        for pattern in &[" .. ", " ... "] {
            let mut search_start = 0usize;
            while let Some(pos) = src[search_start..].find(pattern) {
                let abs_pos = search_start + pos;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Avoid spaces inside range literals.".into(),
                    range: TextRange::new(abs_pos as u32, (abs_pos + pattern.len()) as u32),
                    severity: Severity::Warning,
                });
                search_start = abs_pos + pattern.len();
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantSplatExpansion;

impl Rule for RedundantSplatExpansion {
    fn name(&self) -> &'static str {
        "Lint/RedundantSplatExpansion"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        // Detect `*[` pattern (splat on array literal)
        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find("*[") {
            let abs_pos = search_start + pos;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Redundant splat expansion on array literal `*[...]`.".into(),
                range: TextRange::new(abs_pos as u32, (abs_pos + 2) as u32),
                severity: Severity::Warning,
            });
            search_start = abs_pos + 2;
        }

        diags
    }
}

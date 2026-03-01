use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct EmptyInterpolation;

impl Rule for EmptyInterpolation {
    fn name(&self) -> &'static str {
        "Lint/EmptyInterpolation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find("${}") {
            // Only look for `#{}` which is the actual empty interpolation
            let abs_pos = search_start + pos;
            search_start = abs_pos + 3;
        }

        // Reset and look for `#{}`
        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find("#{}") {
            let abs_pos = search_start + pos;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Empty string interpolation `#{}` detected.".into(),
                range: TextRange::new(abs_pos as u32, (abs_pos + 3) as u32),
                severity: Severity::Warning,
            });
            search_start = abs_pos + 3;
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: String::new(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceBeforeSemicolon;

impl Rule for SpaceBeforeSemicolon {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeSemicolon"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find(" ;") {
            let abs_pos = search_start + pos;
            // Make sure this isn't inside a string
            // Simple: just flag all occurrences
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Avoid space before semicolon.".into(),
                range: TextRange::new(abs_pos as u32, (abs_pos + 1) as u32),
                severity: Severity::Warning,
            });
            search_start = abs_pos + 2;
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

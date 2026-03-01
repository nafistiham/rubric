use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceInLambdaLiteral;

impl Rule for SpaceInLambdaLiteral {
    fn name(&self) -> &'static str {
        "Layout/SpaceInLambdaLiteral"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find("-> (") {
            let abs_pos = search_start + pos;
            // The space is between `->` and `(`
            let space_pos = abs_pos + 2; // position of the space
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Avoid space between `->` and `(` in lambda literal.".into(),
                range: TextRange::new(space_pos as u32, (space_pos + 1) as u32),
                severity: Severity::Warning,
            });
            search_start = abs_pos + "-> (".len();
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

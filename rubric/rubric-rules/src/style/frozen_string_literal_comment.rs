use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct FrozenStringLiteralComment;

impl Rule for FrozenStringLiteralComment {
    fn name(&self) -> &'static str {
        "Style/FrozenStringLiteralComment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let has_comment = ctx
            .lines
            .first()
            .map(|l| l.trim() == "# frozen_string_literal: true")
            .unwrap_or(false);
        if !has_comment {
            vec![Diagnostic {
                rule: self.name(),
                message: "Missing frozen string literal comment.".into(),
                range: TextRange::new(0, 0),
                severity: Severity::Warning,
            }]
        } else {
            vec![]
        }
    }

    fn fix(&self, _diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: TextRange::new(0, 0),
                replacement: "# frozen_string_literal: true\n\n".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

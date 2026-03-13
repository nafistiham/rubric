use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct FrozenStringLiteralComment;

impl Rule for FrozenStringLiteralComment {
    fn name(&self) -> &'static str {
        "Style/FrozenStringLiteralComment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        // Empty files don't need a frozen string literal comment.
        if ctx.lines.is_empty() {
            return vec![];
        }
        // RuboCop allows the comment on line 2 when line 1 is a shebang or encoding comment.
        let mut idx = 0;
        if let Some(first) = ctx.lines.first() {
            let t = first.trim();
            if t.starts_with("#!") || t.starts_with("# encoding:") || t.starts_with("# -*- encoding") {
                idx = 1;
            }
        }
        let has_comment = ctx.lines.get(idx)
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

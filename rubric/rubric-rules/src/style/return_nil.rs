use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct ReturnNil;

impl Rule for ReturnNil {
    fn name(&self) -> &'static str {
        "Style/ReturnNil"
    }

    // RuboCop ships Style/ReturnNil with `Enabled: false`. Match that default
    // so rubric does not flag it unless the user opts in via rubric.toml.
    fn default_enabled(&self) -> bool {
        false
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find("return nil") {
            let abs_pos = search_start + pos;
            // Check that `return nil` is followed by end of statement (space, newline, or end)
            let after_pos = abs_pos + "return nil".len();
            let next_char = src.as_bytes().get(after_pos).copied();
            let is_end = matches!(next_char, None | Some(b'\n') | Some(b' ') | Some(b'\r') | Some(b';'));

            // Also check that `return` is preceded by word boundary
            let before_ok = abs_pos == 0 || {
                let b = src.as_bytes()[abs_pos - 1];
                !b.is_ascii_alphanumeric() && b != b'_'
            };

            if is_end && before_ok {
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `return` instead of `return nil`.".into(),
                    range: TextRange::new(abs_pos as u32, after_pos as u32),
                    severity: Severity::Warning,
                });
            }

            search_start = abs_pos + "return nil".len();
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: "return".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

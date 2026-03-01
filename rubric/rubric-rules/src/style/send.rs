use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct Send;

impl Rule for Send {
    fn name(&self) -> &'static str {
        "Style/Send"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find(".send(") {
            let abs_pos = search_start + pos;

            // Exclude `__send__` — check 8 chars before `.`
            let check_start = abs_pos.saturating_sub(8);
            let before_ok = !src[check_start..abs_pos].ends_with("__");

            if before_ok {
                let method_start = abs_pos + 1; // skip the `.`
                let method_end = abs_pos + ".send(".len();
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `public_send` instead of `send` to avoid calling private methods.".into(),
                    range: TextRange::new(method_start as u32, (method_end - 1) as u32),
                    severity: Severity::Warning,
                });
            }

            search_start = abs_pos + ".send(".len();
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: "public_send".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

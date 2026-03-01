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

            // Exclude `__send__(` and `public_send(`
            let before_ok = abs_pos == 0 || {
                // Make sure this isn't `__send__` — check 8 chars before `.`
                let check_start = if abs_pos >= 8 { abs_pos - 8 } else { 0 };
                !src[check_start..abs_pos].ends_with("__")
            };

            // Ensure `.send(` is not part of `public_send(`
            let is_public_send = abs_pos >= 7 && src[abs_pos - 6..abs_pos].ends_with("public");

            if before_ok && !is_public_send {
                let method_start = abs_pos + 1; // skip the `.`
                let method_end = abs_pos + ".send(".len() as usize;
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

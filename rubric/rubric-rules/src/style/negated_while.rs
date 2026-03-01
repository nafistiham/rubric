use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct NegatedWhile;

impl Rule for NegatedWhile {
    fn name(&self) -> &'static str {
        "Style/NegatedWhile"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let offending = if trimmed.starts_with("while !") {
                Some(("while !", "until "))
            } else if trimmed.starts_with("until !") {
                Some(("until !", "while "))
            } else {
                None
            };

            if let Some((_pattern, _replacement)) = offending {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                let kw_end = pos + 5; // length of "while" or "until"
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: if trimmed.starts_with("while !") {
                        "Use `until` instead of `while !`.".into()
                    } else {
                        "Use `while` instead of `until !`.".into()
                    },
                    range: TextRange::new(pos, kw_end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // Fix replaces the keyword+negation with the inverted keyword
        // We cannot derive the exact replacement text from Diagnostic alone without the source,
        // so we return None here. A full fix would need source context.
        let _ = diag;
        None
    }
}

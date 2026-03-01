use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct PreferredHashMethods;

impl Rule for PreferredHashMethods {
    fn name(&self) -> &'static str {
        "Style/PreferredHashMethods"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        for (old, msg) in &[
            (".has_key?(", "Use `key?` instead of `has_key?`."),
            (".has_value?(", "Use `value?` instead of `has_value?`."),
        ] {
            let mut search_start = 0usize;
            while let Some(pos) = src[search_start..].find(old) {
                let abs_pos = search_start + pos;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: msg.to_string(),
                    range: TextRange::new(
                        (abs_pos + 1) as u32, // skip the `.`
                        (abs_pos + old.len()) as u32,
                    ),
                    severity: Severity::Warning,
                });
                search_start = abs_pos + old.len();
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        let _ = diag;
        None
    }
}

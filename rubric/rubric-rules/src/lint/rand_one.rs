use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RandOne;

impl Rule for RandOne {
    fn name(&self) -> &'static str {
        "Lint/RandOne"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        let mut search_start = 0usize;
        while let Some(pos) = src[search_start..].find("rand(1)") {
            let abs_pos = search_start + pos;
            // Word boundary before `rand`
            let before_ok = abs_pos == 0 || {
                let b = src.as_bytes()[abs_pos - 1];
                !b.is_ascii_alphanumeric() && b != b'_'
            };

            if before_ok {
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "`rand(1)` always returns 0; did you mean `rand(2)` or another value?".into(),
                    range: TextRange::new(abs_pos as u32, (abs_pos + "rand(1)".len()) as u32),
                    severity: Severity::Warning,
                });
            }

            search_start = abs_pos + "rand(1)".len();
        }

        diags
    }
}

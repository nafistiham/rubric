use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct BooleanSymbol;

impl Rule for BooleanSymbol {
    fn name(&self) -> &'static str {
        "Lint/BooleanSymbol"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        for pattern in &[":true", ":false"] {
            let mut search_start = 0usize;
            while let Some(pos) = src[search_start..].find(pattern) {
                let abs_pos = search_start + pos;
                // Check word boundary after pattern
                let end = abs_pos + pattern.len();
                let after_ok = end >= src.len() || {
                    let b = src.as_bytes()[end];
                    !b.is_ascii_alphanumeric() && b != b'_'
                };
                // Check that `:` is not preceded by `:` (avoid `::true`) or a quote char
                let before_ok = abs_pos == 0 || {
                    let before = src.as_bytes()[abs_pos - 1];
                    before != b':' && before != b'"' && before != b'\''
                };

                if after_ok && before_ok {
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "`{}` is likely a bug; use `{}` (boolean) instead.",
                            pattern,
                            &pattern[1..]
                        ),
                        range: TextRange::new(abs_pos as u32, end as u32),
                        severity: Severity::Warning,
                    });
                }
                search_start = abs_pos + pattern.len();
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UriEscapeUnescape;

impl Rule for UriEscapeUnescape {
    fn name(&self) -> &'static str {
        "Lint/UriEscapeUnescape"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        for pattern in &["URI.escape", "URI.unescape"] {
            let mut search_start = 0usize;
            while let Some(pos) = src[search_start..].find(pattern) {
                let abs_pos = search_start + pos;
                let end = abs_pos + pattern.len();
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "`{}` is deprecated; use `URI::DEFAULT_PARSER` methods instead.",
                        pattern
                    ),
                    range: TextRange::new(abs_pos as u32, end as u32),
                    severity: Severity::Warning,
                });
                search_start = end;
            }
        }

        diags
    }
}

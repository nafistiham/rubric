use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct DeprecatedClassMethods;

impl Rule for DeprecatedClassMethods {
    fn name(&self) -> &'static str {
        "Lint/DeprecatedClassMethods"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;

        for (old, new, msg) in &[
            ("File.exists?", "File.exist?", "Use `File.exist?` instead of deprecated `File.exists?`."),
            ("Dir.exists?", "Dir.exist?", "Use `Dir.exist?` instead of deprecated `Dir.exists?`."),
            ("Thread.exclusive", "Mutex#synchronize", "Use `Mutex#synchronize` instead of deprecated `Thread.exclusive`."),
        ] {
            let mut search_start = 0usize;
            while let Some(pos) = src[search_start..].find(old) {
                let abs_pos = search_start + pos;
                let end = abs_pos + old.len();
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: msg.to_string(),
                    range: TextRange::new(abs_pos as u32, end as u32),
                    severity: Severity::Warning,
                });
                search_start = end;
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        let _ = diag;
        None
    }
}

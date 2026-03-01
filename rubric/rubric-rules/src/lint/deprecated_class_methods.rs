use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DeprecatedClassMethods;

impl Rule for DeprecatedClassMethods {
    fn name(&self) -> &'static str {
        "Lint/DeprecatedClassMethods"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (old, msg) in &[
            ("File.exists?", "Use `File.exist?` instead of deprecated `File.exists?`."),
            ("Dir.exists?", "Use `Dir.exist?` instead of deprecated `Dir.exists?`."),
            ("Thread.exclusive", "Use `Mutex#synchronize` instead of deprecated `Thread.exclusive`."),
        ] {
            for (i, line) in ctx.lines.iter().enumerate() {
                let trimmed = line.trim_start();
                // Skip comment lines
                if trimmed.starts_with('#') {
                    continue;
                }

                let line_start = ctx.line_start_offsets[i] as usize;
                let mut search_start = 0usize;

                while let Some(pos) = line[search_start..].find(old) {
                    let pos_in_line = search_start + pos;
                    // Skip if inside a string: check if there's an unmatched quote before position
                    let prefix = &line[..pos_in_line];
                    if prefix.contains('"') || prefix.contains('\'') {
                        search_start = pos_in_line + old.len();
                        continue;
                    }
                    // Skip if there's a comment character before the match
                    if prefix.contains('#') {
                        break;
                    }

                    let abs_pos = line_start + pos_in_line;
                    let end = abs_pos + old.len();
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: msg.to_string(),
                        range: TextRange::new(abs_pos as u32, end as u32),
                        severity: Severity::Warning,
                    });
                    search_start = pos_in_line + old.len();
                }
            }
        }

        diags
    }
}

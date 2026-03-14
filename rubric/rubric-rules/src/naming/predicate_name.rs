use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct PredicateName;

const FORBIDDEN_PREFIXES: &[&str] = &["is_", "has_", "have_"];

impl Rule for PredicateName {
    fn name(&self) -> &'static str {
        "Naming/PredicateName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                continue;
            }

            // Must start with `def `
            let after_def = if let Some(rest) = trimmed.strip_prefix("def ") {
                rest
            } else if let Some(rest) = trimmed.strip_prefix("def\t") {
                rest
            } else {
                continue;
            };

            // Skip self. prefix
            let method_part = after_def.strip_prefix("self.").unwrap_or(after_def);

            // Extract method name up to (, space, tab, or end
            let name_end = method_part
                .find(|c: char| c == '(' || c == ' ' || c == '\t' || c == '\n')
                .unwrap_or(method_part.len());
            let method_name = &method_part[..name_end];

            if method_name.is_empty() {
                continue;
            }

            // Must end with `?` to be a predicate
            if !method_name.ends_with('?') {
                continue;
            }

            // Check for forbidden prefixes
            let bad_prefix = FORBIDDEN_PREFIXES
                .iter()
                .find(|&&prefix| method_name.starts_with(prefix));

            if let Some(prefix) = bad_prefix {
                // Suggest the name without the prefix
                let suggested = &method_name[prefix.len()..];
                let line_start = ctx.line_start_offsets[i] as usize;
                let def_col = line.len() - trimmed.len();
                let start = (line_start + def_col) as u32;
                let end = start + 3; // `def`
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "Rename `{}` to `{}`.",
                        method_name, suggested
                    ),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

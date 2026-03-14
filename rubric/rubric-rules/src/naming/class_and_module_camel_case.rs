use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ClassAndModuleCamelCase;

impl Rule for ClassAndModuleCamelCase {
    fn name(&self) -> &'static str {
        "Naming/ClassAndModuleCamelCase"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Determine if this line opens a class or module definition
            let name_start = if trimmed.starts_with("class ") {
                "class ".len()
            } else if trimmed.starts_with("module ") {
                "module ".len()
            } else {
                continue;
            };

            let after_keyword = &trimmed[name_start..];

            // Skip `class << self` (singleton class opener)
            if after_keyword.trim_start().starts_with("<<") {
                continue;
            }

            // Extract the name: stop at `<`, `;`, `(`, whitespace, or end of string
            let name: &str = {
                let end = after_keyword
                    .find(|c: char| c == '<' || c == ';' || c == '(' || c.is_whitespace())
                    .unwrap_or(after_keyword.len());
                &after_keyword[..end]
            };

            if name.is_empty() {
                continue;
            }

            // Flag if name starts with a lowercase letter or contains an underscore
            let starts_lowercase = name
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase())
                .unwrap_or(false);
            let has_underscore = name.contains('_');

            if starts_lowercase || has_underscore {
                let line_start = ctx.line_start_offsets[i] as usize;
                let indent = line.len() - trimmed.len();
                // Point at the name itself (after the keyword + space)
                let col = indent + name_start;
                let start = (line_start + col) as u32;
                let end = (line_start + col + name.len()) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use CamelCase for class and module names.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

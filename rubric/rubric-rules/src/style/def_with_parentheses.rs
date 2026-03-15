use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DefWithParentheses;

impl Rule for DefWithParentheses {
    fn name(&self) -> &'static str {
        "Style/DefWithParentheses"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines.
            if trimmed.starts_with('#') {
                continue;
            }

            // Must start with `def ` or `def\t`.
            let after_def = if let Some(rest) = trimmed.strip_prefix("def ") {
                rest
            } else if let Some(rest) = trimmed.strip_prefix("def\t") {
                rest
            } else {
                continue;
            };

            // Find the method name. Method names may include alphanumeric, `_`,
            // and optionally a `?`, `!`, or `=` suffix.
            // Scan until we hit `(`, space, tab, newline, or end of string.
            let name_end = after_def
                .find(|c: char| c == '(' || c == ' ' || c == '\t' || c == '\n')
                .unwrap_or(after_def.len());

            let method_name = &after_def[..name_end];

            if method_name.is_empty() {
                continue;
            }

            let rest_after_name = &after_def[name_end..];

            // Flag only when the parens are immediately after the name and
            // contain nothing: `def foo()`.
            if !rest_after_name.starts_with("()") {
                continue;
            }

            // Compute position of the `(` in the source.
            let indent = line.len() - trimmed.len();
            // `def ` is 4 bytes; method name follows.
            let paren_col = indent + "def ".len() + method_name.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let start = (line_start + paren_col) as u32;
            let end = start + 1; // just the `(`

            diags.push(Diagnostic {
                rule: self.name(),
                message: "Omit the parentheses in defs when the method doesn't accept any arguments."
                    .into(),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

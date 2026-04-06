use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MethodDefParentheses;

/// Returns true if `name` is an operator method that should be skipped.
fn is_operator_method(name: &str) -> bool {
    matches!(
        name,
        "==" | "!=" | "===" | "<=>" | "<" | ">" | "<=" | ">=" | "[]" | "[]=" | "+"
            | "-" | "*" | "/" | "%" | "**" | "<<" | ">>" | "&" | "|" | "^" | "~"
            | "+@" | "-@" | "!" | "!~" | "=~"
    )
}

impl Rule for MethodDefParentheses {
    fn name(&self) -> &'static str {
        "Style/MethodDefParentheses"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Must start with `def ` (or be `def\t` etc.)
            let after_def = if let Some(rest) = trimmed.strip_prefix("def ") {
                rest
            } else if let Some(rest) = trimmed.strip_prefix("def\t") {
                rest
            } else {
                continue;
            };

            // Skip one-liner methods (contains `; end`)
            if line.contains("; end") || line.contains(";end") {
                continue;
            }

            // Extract the method name — it's everything up to `(`, ` `, `;`, or end of token
            let name_end = after_def
                .find(|c: char| c == '(' || c == ' ' || c == '\t' || c == '\n' || c == ';')
                .unwrap_or(after_def.len());
            let method_name = &after_def[..name_end];

            if method_name.is_empty() {
                continue;
            }

            // Skip operator methods
            if is_operator_method(method_name) {
                continue;
            }

            let rest_after_name = &after_def[name_end..];

            // Skip endless method definitions: `def foo = expr` (Ruby 3.0+).
            // The `=` here is the body assignment, not a parameter list.
            let trimmed_rn = rest_after_name.trim_start();
            if trimmed_rn.starts_with('=') && !trimmed_rn.starts_with("==") {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            // Offset of `def` in the line
            let def_col = line.len() - trimmed.len();
            let start = (line_start + def_col) as u32;
            let end = start + 3; // `def`

            if rest_after_name.starts_with('(') {
                // Has parentheses — check if empty: `()`
                if rest_after_name.starts_with("()") {
                    // Empty parens on a def with no params — flag it
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Omit parentheses in method definition.".into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                }
                // else: has params in parens — fine
            } else {
                // No parentheses at all after method name
                let trimmed_rest = rest_after_name.trim_start();
                if !trimmed_rest.is_empty()
                    && !trimmed_rest.starts_with('#')
                    && !trimmed_rest.starts_with('\n')
                    && !trimmed_rest.starts_with(';')  // inline body, not params
                {
                    // There's something after the method name — those are params without parens
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use parentheses in method definition.".into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                }
                // else: no params, no parens — that's fine
            }
        }

        diags
    }
}

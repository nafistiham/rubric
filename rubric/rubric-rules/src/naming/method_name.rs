use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MethodName;

/// Returns true if the method name is an operator that should be skipped.
fn is_operator(name: &str) -> bool {
    matches!(
        name,
        "==" | "!=" | "===" | "<=>" | "<" | ">" | "<=" | ">=" | "[]" | "[]=" | "+"
            | "-" | "*" | "/" | "%" | "**" | "<<" | ">>" | "&" | "|" | "^" | "~"
            | "+@" | "-@" | "!" | "!~" | "=~" | "to_s" | "to_i" | "to_a" | "to_h"
    )
}

/// Returns true if the method name is in snake_case.
/// Allowed: lowercase letters, digits, underscores, optionally ending with `?`, `!`, or `=`.
fn is_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    // Strip trailing modifier suffix
    let base = name
        .strip_suffix('?')
        .or_else(|| name.strip_suffix('!'))
        .or_else(|| name.strip_suffix('='))
        .unwrap_or(name);

    // All chars must be lowercase letter, digit, underscore, or non-ASCII (Unicode identifiers)
    base.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || !c.is_ascii())
}

impl Rule for MethodName {
    fn name(&self) -> &'static str {
        "Naming/MethodName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                continue;
            }

            // Must contain `def ` or `def\t`
            let after_def = if let Some(rest) = trimmed.strip_prefix("def ") {
                rest
            } else if let Some(rest) = trimmed.strip_prefix("def\t") {
                rest
            } else {
                continue;
            };

            // Extract raw token: up to `(`, ` `, `\t`, `;`, or end
            let name_end = after_def
                .find(|c: char| c == '(' || c == ' ' || c == '\t' || c == '\n' || c == ';')
                .unwrap_or(after_def.len());
            let raw_name = &after_def[..name_end];

            // Skip dynamically generated method names (inside class_eval heredocs / string templates)
            if raw_name.contains("#{") {
                continue;
            }

            // Strip receiver prefix: `self.method`, `obj.method` → take the part after the last `.`
            let method_name = if let Some(dot) = raw_name.rfind('.') {
                &raw_name[dot + 1..]
            } else {
                raw_name
            };

            if method_name.is_empty() {
                continue;
            }

            // Skip operators
            if is_operator(method_name) {
                continue;
            }

            if !is_snake_case(method_name) {
                let line_start = ctx.line_start_offsets[i] as usize;
                let def_col = line.len() - trimmed.len();
                let start = (line_start + def_col) as u32;
                let end = start + 3; // `def`
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "Use snake_case for method names (`{}`).",
                        method_name
                    ),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

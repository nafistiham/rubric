use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct BinaryOperatorParameterName;

/// Binary operators whose parameter should be named `other`.
/// Ordered so that multi-char operators are checked before single-char prefixes.
/// Binary operators whose parameter should be named `other`.
/// Ordered so that multi-char operators are checked before single-char prefixes.
/// NOTE: `[]` and `[]=` are NOT binary operators — they are indexer methods
/// whose parameter is conventionally named `key`, `index`, etc. Rubocop does
/// not enforce `other` for these.
const BINARY_OPS: &[&str] = &[
    "<=>", "**", "!=", "<=", ">=", "<<", ">>", "==", "+", "-", "*", "/", "%", "&", "|", "^",
    "<", ">",
];

impl Rule for BinaryOperatorParameterName {
    fn name(&self) -> &'static str {
        "Naming/BinaryOperatorParameterName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Must contain `def ` or `def\t`
            let after_def = if let Some(rest) = trimmed.strip_prefix("def ") {
                rest.trim_start()
            } else if let Some(rest) = trimmed.strip_prefix("def\t") {
                rest.trim_start()
            } else {
                continue;
            };

            // Try to match a binary operator at the start of `after_def`
            let Some((op, rest_after_op)) = match_operator(after_def) else {
                continue;
            };

            // After the operator there must be `(param)` optionally with whitespace
            let rest_trimmed = rest_after_op.trim_start();
            let Some(inner) = rest_trimmed
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')').or_else(|| strip_to_close_paren(s)))
            else {
                continue;
            };

            let param = inner.trim();

            // Param must be a plain identifier (not empty, not `other`)
            if param.is_empty() || param == "other" {
                continue;
            }

            // Emit a diagnostic pointing at the `def` keyword
            let line_start = ctx.line_start_offsets[i] as usize;
            let def_col = line.len() - trimmed.len();
            let start = (line_start + def_col) as u32;
            let end = start + 3; // length of `def`

            diags.push(Diagnostic {
                rule: self.name(),
                message: format!(
                    "When defining the '{}' operator, name its argument other.",
                    op
                ),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

/// Try to match a binary operator at the start of `s`.
/// Returns `(operator_str, remainder_after_operator)` on success.
fn match_operator(s: &str) -> Option<(&'static str, &str)> {
    for &op in BINARY_OPS {
        if s.starts_with(op) {
            // Make sure the operator is not followed by another operator character
            // (e.g. `==` should not match inside `===`)
            let rest = &s[op.len()..];
            let next = rest.chars().next();
            match next {
                // `[]=` is already a full operator — allow immediately
                Some('(') | Some(' ') | Some('\t') | None => {
                    return Some((op, rest));
                }
                _ => {
                    // e.g. `===` after matching `==` — skip
                    continue;
                }
            }
        }
    }
    None
}

/// Given a string that starts after the opening `(`, find the content up to
/// the first `)` and return it (ignoring any trailing content).
fn strip_to_close_paren(s: &str) -> Option<&str> {
    let end = s.find(')')?;
    Some(&s[..end])
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct LiteralAsCondition;

/// Keywords that introduce a condition followed by an expression.
const CONDITION_KEYWORDS: &[&str] = &["if ", "unless ", "while ", "until "];

/// Extract heredoc terminator word from a line (e.g. `<<TXT`, `<<~SQL`, `<<-'EOF'`).
fn heredoc_terminator_word(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let rest = &line[i + 2..];
            let rest = rest.strip_prefix('-').or_else(|| rest.strip_prefix('~')).unwrap_or(rest);
            let rest = if rest.starts_with('"') || rest.starts_with('\'') || rest.starts_with('`') {
                &rest[1..]
            } else {
                rest
            };
            let word: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !word.is_empty() {
                return Some(word);
            }
        }
        i += 1;
    }
    None
}

/// Returns `Some(literal_str)` if `expr` starts with a literal value, or `None`.
///
/// Recognised literals:
/// - `nil`, `true`, `false`
/// - Integer: one or more ASCII digits (optionally with `_` separators)
/// - Float: digits `.` digits
/// - Single-quoted string: starts with `'`
/// - Double-quoted string: starts with `"`
/// - Symbol: starts with `:` followed by a word character
fn extract_leading_literal<'a>(expr: &'a str) -> Option<&'a str> {
    let expr = expr.trim_start();

    // Keyword literals
    for kw in &["nil", "true", "false"] {
        if expr.starts_with(kw) {
            // Ensure the keyword is not a prefix of an identifier
            let rest = &expr[kw.len()..];
            let boundary = rest
                .chars()
                .next()
                .map(|c| !c.is_alphanumeric() && c != '_')
                .unwrap_or(true);
            if boundary {
                return Some(kw);
            }
        }
    }

    // Float: must be tested before integer (more specific pattern)
    if expr.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        // Collect digits
        let int_end = expr
            .char_indices()
            .find(|(_, c)| !c.is_ascii_digit() && *c != '_')
            .map(|(i, _)| i)
            .unwrap_or(expr.len());
        if int_end < expr.len() && expr.as_bytes()[int_end] == b'.' {
            let frac_start = int_end + 1;
            let frac_end = expr[frac_start..]
                .char_indices()
                .find(|(_, c)| !c.is_ascii_digit())
                .map(|(i, _)| frac_start + i)
                .unwrap_or(expr.len());
            if frac_end > frac_start {
                return Some(&expr[..frac_end]);
            }
        }
        return Some(&expr[..int_end]);
    }

    // String literals: single-quoted
    if expr.starts_with('\'') {
        return Some("'...'");
    }

    // String literals: double-quoted
    if expr.starts_with('"') {
        return Some("\"...\"");
    }

    // Symbol: `:word`
    if expr.starts_with(':') {
        let rest = &expr[1..];
        if rest.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
            let sym_end = rest
                .char_indices()
                .find(|(_, c)| !c.is_alphanumeric() && *c != '_')
                .map(|(i, _)| i)
                .unwrap_or(rest.len());
            return Some(&expr[..sym_end + 1]);
        }
    }

    // Regex literal: `/pattern/`
    if expr.starts_with('/') {
        return Some("/regex/");
    }

    None
}

impl Rule for LiteralAsCondition {
    fn name(&self) -> &'static str {
        "Lint/LiteralAsCondition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc: Option<String> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // Detect heredoc opening — body starts on NEXT line
            if let Some(term) = heredoc_terminator_word(line) {
                in_heredoc = Some(term);
                // Fall through: still check the opener line itself
            }

            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as u32;

            for &kw in CONDITION_KEYWORDS {
                if trimmed.starts_with(kw) {
                    let condition_expr = &trimmed[kw.len()..];
                    if let Some(literal) = extract_leading_literal(condition_expr) {
                        // Skip if the literal is used as a receiver (followed by `.`)
                        // e.g. `if 5.class.name == 'Integer'` — `5` is a receiver, not the condition
                        let after_literal = condition_expr[literal.len()..].trim_start();
                        if after_literal.starts_with('.') {
                            break;
                        }
                        let line_end = line_start + line.len() as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!("Literal `{}` appeared as a condition.", literal),
                            range: TextRange::new(line_start, line_end),
                            severity: Severity::Warning,
                        });
                    }
                    break;
                }
            }
        }

        diags
    }
}

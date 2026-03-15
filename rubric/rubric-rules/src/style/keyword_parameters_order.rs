use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct KeywordParametersOrder;

impl Rule for KeywordParametersOrder {
    fn name(&self) -> &'static str {
        "Style/KeywordParametersOrder"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Only check lines with `def `
            if !line.contains("def ") {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;

            // Extract parameter list between first `(` and matching `)`
            let param_str = match extract_param_list(line) {
                Some(s) => s,
                None => continue,
            };

            if has_ordering_violation(&param_str) {
                // Point the diagnostic at the `def` keyword
                let def_pos = line.find("def ").unwrap_or(0);
                let start = (line_start + def_pos) as u32;
                let end = (line_start + line.len()) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message:
                        "Required keyword arguments should be placed before optional keyword arguments."
                            .to_string(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Extracts the content between the first `(` and its matching `)` on the line.
/// Returns `None` if no parentheses found or they are unbalanced.
fn extract_param_list(line: &str) -> Option<&str> {
    let open = line.find('(')?;
    let after_open = &line[open + 1..];
    // Find matching close — simple single-level balance (method def params are flat)
    let mut depth = 1usize;
    let mut end_rel = None;
    for (idx, ch) in after_open.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end_rel = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let end_rel = end_rel?;
    Some(&after_open[..end_rel])
}

/// Parses keyword parameters from the param list string and returns true if any optional
/// keyword param appears before a required keyword param.
///
/// A keyword param is a token of the form `word:` (required) or `word: <expr>` (optional).
/// We detect them by finding `:` that follows an identifier character and is not preceded
/// by another `:` (to skip symbol literals like `:foo`).
fn has_ordering_violation(params: &str) -> bool {
    let keyword_params = collect_keyword_params(params);

    // Walk through: once we see an optional, any subsequent required is a violation
    let mut saw_optional = false;
    for kp in &keyword_params {
        if kp.optional {
            saw_optional = true;
        } else if saw_optional {
            // Required after optional — violation
            return true;
        }
    }
    false
}

#[derive(Debug)]
struct KeywordParam {
    optional: bool,
}

/// Splits `params` by commas (top-level only) and identifies keyword params.
fn collect_keyword_params(params: &str) -> Vec<KeywordParam> {
    let segments = split_top_level_commas(params);
    let mut result = Vec::new();

    for seg in segments {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        if let Some(kp) = classify_param(seg) {
            result.push(kp);
        }
    }
    result
}

/// Classifies a single parameter segment.
/// Returns `Some(KeywordParam)` if it looks like a keyword param, `None` otherwise.
fn classify_param(seg: &str) -> Option<KeywordParam> {
    // A keyword parameter contains a bare `:` after an identifier.
    // Forms:
    //   required:  `foo:` (identifier followed by `:` then end or whitespace or `,`)
    //   optional:  `foo: default_expr`
    //
    // We find the first `:` that is:
    //   - preceded by an identifier character (alphanumeric or `_`)
    //   - not preceded by another `:` (that would be `::`)
    //   - not followed by `:` (that would be `::`)

    let bytes = seg.as_bytes();
    let n = bytes.len();
    let mut colon_pos: Option<usize> = None;

    for j in 0..n {
        if bytes[j] == b':' {
            // Check it's not `::` (double colon)
            let prev_colon = j > 0 && bytes[j - 1] == b':';
            let next_colon = j + 1 < n && bytes[j + 1] == b':';
            if prev_colon || next_colon {
                continue;
            }
            // Preceded by identifier char
            if j > 0 {
                let pb = bytes[j - 1];
                if pb.is_ascii_alphanumeric() || pb == b'_' {
                    colon_pos = Some(j);
                    break;
                }
            }
        }
    }

    let colon_pos = colon_pos?;

    // Everything before the colon should be a valid identifier (possibly prefixed with `*` or `**`)
    // Anything after the colon (trimmed) — if non-empty, it's optional; if empty, it's required.
    let after_colon = seg[colon_pos + 1..].trim();

    // Skip double-splat keyword rest (`**opts` — not a keyword param)
    if seg.starts_with("**") {
        return None;
    }

    let optional = !after_colon.is_empty();
    Some(KeywordParam { optional })
}

/// Splits `s` by commas that are not inside `()`, `[]`, or `{}`.
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    let bytes = s.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            b',' if depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&s[start..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn check(source: &str) -> Vec<Diagnostic> {
        let ctx = LintContext::new(Path::new("test.rb"), source);
        KeywordParametersOrder.check_source(&ctx)
    }

    // --- Violation cases ---

    #[test]
    fn flags_optional_before_required() {
        let diags = check("def foo(bar: 1, baz:)\nend\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "Style/KeywordParametersOrder");
        assert!(diags[0].message.contains("Required keyword"));
    }

    #[test]
    fn flags_optional_before_required_multiple() {
        // `a:` required, `b: 2` optional, `c:` required — violation
        let diags = check("def bar(a:, b: 2, c:)\nend\n");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn flags_first_optional_then_required() {
        let diags = check("def method(opt: 'default', req:)\nend\n");
        assert_eq!(diags.len(), 1);
    }

    // --- Good cases (no violation) ---

    #[test]
    fn no_flag_required_then_optional() {
        let diags = check("def foo(baz:, bar: 1)\nend\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_all_required() {
        let diags = check("def foo(a:, b:, c:)\nend\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_all_optional() {
        let diags = check("def foo(a: 1, b: 2, c: 3)\nend\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_no_keyword_params() {
        let diags = check("def foo(a, b, c)\nend\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_no_parens() {
        let diags = check("def foo\nend\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_comment_line() {
        let diags = check("# def foo(bar: 1, baz:)\nend\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_non_def_line() {
        let diags = check("foo(bar: 1, baz: 2)\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_mixed_positional_and_keyword_correct_order() {
        let diags = check("def foo(pos, req:, opt: 42)\nend\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn flags_mixed_positional_and_keyword_wrong_order() {
        let diags = check("def foo(pos, opt: 42, req:)\nend\n");
        assert_eq!(diags.len(), 1);
    }

    // --- Unit tests for helpers ---

    #[test]
    fn extract_param_list_simple() {
        assert_eq!(extract_param_list("def foo(a, b)"), Some("a, b"));
    }

    #[test]
    fn extract_param_list_no_parens() {
        assert_eq!(extract_param_list("def foo"), None);
    }

    #[test]
    fn split_top_level_commas_nested() {
        let parts = split_top_level_commas("a, b: foo(1, 2), c:");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].trim(), "a");
        assert_eq!(parts[1].trim(), "b: foo(1, 2)");
        assert_eq!(parts[2].trim(), "c:");
    }

    #[test]
    fn classify_param_required_keyword() {
        let kp = classify_param("foo:").unwrap();
        assert!(!kp.optional);
    }

    #[test]
    fn classify_param_optional_keyword() {
        let kp = classify_param("foo: 42").unwrap();
        assert!(kp.optional);
    }

    #[test]
    fn classify_param_positional_returns_none() {
        assert!(classify_param("foo").is_none());
    }
}

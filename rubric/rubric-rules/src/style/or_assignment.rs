use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct OrAssignment;

/// Extract a leading variable name from a slice (may include `@`, `@@` prefix,
/// then alphanumeric/underscore chars). Returns the variable string or None.
fn extract_var(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut i = 0;

    // Allow `@@` or `@` prefix
    if bytes.get(i).copied() == Some(b'@') {
        i += 1;
        if bytes.get(i).copied() == Some(b'@') {
            i += 1;
        }
    }

    let start = i;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }

    if i > start {
        Some(&s[..i])
    } else {
        None
    }
}

/// Find the position of a bare `=` (not `==`, `!=`, `<=`, `>=`, `||=`, `&&=`,
/// `+=`, `-=`, `*=`, `/=`, etc.) in `s`, returning its byte index.
/// Scans outside strings and comments.
fn find_bare_assignment(s: &[u8]) -> Option<usize> {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < s.len() {
        match in_str {
            Some(_) if s[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if s[i] == d => {
                in_str = None;
                i += 1;
                continue;
            }
            Some(_) => {
                i += 1;
                continue;
            }
            None => {}
        }

        match s[i] {
            b'"' | b'\'' => {
                in_str = Some(s[i]);
            }
            b'#' => {
                // Comment — stop scanning
                break;
            }
            b'=' => {
                // Reject `==`
                if s.get(i + 1).copied() == Some(b'=') {
                    i += 2;
                    continue;
                }
                // Reject compound forms: `!=`, `<=`, `>=`, `||=`, `&&=`,
                // `+=`, `-=`, `*=`, `/=`, `%=`, `^=`, `|=`, `&=`, `~=`
                let prev = if i > 0 { s[i - 1] } else { 0 };
                if prev == b'!'
                    || prev == b'<'
                    || prev == b'>'
                    || prev == b'+'
                    || prev == b'-'
                    || prev == b'*'
                    || prev == b'/'
                    || prev == b'%'
                    || prev == b'^'
                    || prev == b'|'
                    || prev == b'&'
                    || prev == b'~'
                {
                    i += 1;
                    continue;
                }
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

impl Rule for OrAssignment {
    fn name(&self) -> &'static str {
        "Style/OrAssignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Find the bare `=` on this line
            let Some(eq_pos) = find_bare_assignment(bytes) else {
                continue;
            };

            // LHS: everything before the `=`, trimmed
            let lhs = line[..eq_pos].trim();

            // Extract the LHS variable name
            let Some(lhs_var) = extract_var(lhs) else {
                continue;
            };

            // LHS must be exactly the variable (no extra chars after the name)
            if lhs != lhs_var {
                continue;
            }

            // RHS: everything after `=`, trimmed
            let rhs = line[eq_pos + 1..].trim();

            // RHS must start with the same variable
            if !rhs.starts_with(lhs_var) {
                continue;
            }

            // The character immediately after the RHS variable name must not be
            // alphanumeric or `_` (avoids matching `foo = foo_bar || x`)
            let rhs_after_var = &rhs[lhs_var.len()..];
            if let Some(first) = rhs_after_var.bytes().next() {
                if first.is_ascii_alphanumeric() || first == b'_' {
                    continue;
                }
            }

            // After the var, the next non-space chars must be `||` (not `||=`)
            let after_var = rhs_after_var.trim_start();
            if !after_var.starts_with("||") {
                continue;
            }
            // Not `||=`
            if after_var.as_bytes().get(2).copied() == Some(b'=') {
                continue;
            }

            let start = (line_start + eq_pos) as u32;
            let end = start + 1;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Use the ||= operator instead of assignment with || operator.".into(),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_x_eq_x_or_value() {
        let src = "x = x || default_value\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = OrAssignment.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for 'x = x || default_value', got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/OrAssignment"));
    }

    #[test]
    fn detects_ivar_eq_ivar_or_value() {
        let src = "@memoized = @memoized || compute\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = OrAssignment.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for '@memoized = @memoized || compute', got none"
        );
    }

    #[test]
    fn detects_cvar_eq_cvar_or_value() {
        let src = "@@count = @@count || 0\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = OrAssignment.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for '@@count = @@count || 0', got none"
        );
    }

    #[test]
    fn no_violation_for_or_assign_operator() {
        let src = "x ||= default_value\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = OrAssignment.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_different_vars() {
        let src = "x = y || default_value\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = OrAssignment.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_multiple_or() {
        let src = "x = a || b || c\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = OrAssignment.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_comment_line() {
        let src = "# x = x || default_value\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = OrAssignment.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations for comment line, got: {:?}",
            diags
        );
    }

    #[test]
    fn no_violation_lhs_prefix_of_rhs_var() {
        // `foo = foo_bar || x` — RHS var is `foo_bar`, not `foo`
        let src = "foo = foo_bar || x\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = OrAssignment.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations for prefix mismatch, got: {:?}",
            diags
        );
    }

    #[test]
    fn uses_offending_fixture() {
        let offending = include_str!("../../tests/fixtures/style/or_assignment/offending.rb");
        let ctx = LintContext::new(Path::new("test.rb"), offending);
        let diags = OrAssignment.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations in offending.rb, got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/OrAssignment"));
    }

    #[test]
    fn no_violation_on_passing_fixture() {
        let passing = include_str!("../../tests/fixtures/style/or_assignment/passing.rb");
        let ctx = LintContext::new(Path::new("test.rb"), passing);
        let diags = OrAssignment.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations in passing.rb, got: {:?}",
            diags
        );
    }
}

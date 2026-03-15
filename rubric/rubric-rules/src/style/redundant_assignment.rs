use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantAssignment;

/// Parse a local variable name from the left-hand side of an assignment line.
/// Returns the variable name if the trimmed line is `<local_var> = <expr>`.
/// Local vars: start with lowercase or `_`, no `@`, `@@`, `$` prefix.
fn parse_local_assignment(line: &str) -> Option<&str> {
    let trimmed = line.trim();

    // Find `=` that is a simple assignment (not `==`, `!=`, `<=`, `>=`, `+=`, etc.)
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' {
            // Reject compound operators: preceding char is `!`, `<`, `>`, `+`, `-`, `*`, `/`, `&`, `|`, `^`, `~`
            let prev_is_op = i > 0 && matches!(
                bytes[i - 1],
                b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'&' | b'|' | b'^' | b'~'
            );
            // Reject `==`
            let next_is_eq = i + 1 < bytes.len() && bytes[i + 1] == b'=';

            if !prev_is_op && !next_is_eq {
                let lhs = trimmed[..i].trim();
                // Must be a simple local variable name: lowercase/underscore start, alphanumeric+underscore only
                if is_local_var(lhs) {
                    return Some(lhs);
                }
                return None;
            }
        }
        i += 1;
    }
    None
}

/// Returns true if `name` is a valid Ruby local variable identifier.
/// Local vars start with a lowercase ASCII letter or `_`, contain only
/// alphanumeric and `_`, and have no sigil prefix.
fn is_local_var(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let bytes = name.as_bytes();
    // Must not start with `@`, `$`, or uppercase
    let first = bytes[0];
    if first == b'@' || first == b'$' || first.is_ascii_uppercase() {
        return false;
    }
    // Must start with lowercase or `_`
    if !first.is_ascii_lowercase() && first != b'_' {
        return false;
    }
    // All chars must be alphanumeric or `_`
    bytes.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_')
}

impl Rule for RedundantAssignment {
    fn name(&self) -> &'static str {
        "Style/RedundantAssignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        let mut i = 0;
        while i + 1 < lines.len() {
            let line_a = lines[i];
            let line_b = lines[i + 1];

            if let Some(var) = parse_local_assignment(line_a) {
                let next_trimmed = line_b.trim();

                // Skip if the assignment line has a trailing conditional modifier
                // (e.g. `cmp = x if cond`) — the assignment is not guaranteed to run.
                let has_trailing_cond = {
                    let t = line_a.trim();
                    // Look for ` if ` or ` unless ` after the `=`
                    let eq_pos = t.find('=').unwrap_or(t.len());
                    let rhs = &t[eq_pos..];
                    rhs.contains(" if ") || rhs.contains(" unless ")
                };

                // Line B must be exactly the variable name (bare return of the var)
                if next_trimmed == var && !has_trailing_cond {
                    // Ensure the variable doesn't appear elsewhere in line_a beyond the LHS
                    // (this prevents false positives when the var is used in the RHS in a
                    // meaningful way — but actually we only want to check it's not used
                    // elsewhere in the method body before this pair).
                    // Simple heuristic: flag unconditionally for the two-line pair.
                    // This matches the plan's specification.
                    let line_offset = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Redundant assignment before returning.".into(),
                        range: TextRange::new(line_offset, line_offset + line_a.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }

            i += 1;
        }

        diags
    }
}

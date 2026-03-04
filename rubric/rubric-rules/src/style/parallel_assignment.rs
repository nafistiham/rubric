use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ParallelAssignment;

/// Returns true if `s` is a simple Ruby literal value:
/// integer, float, string, symbol, boolean, or nil.
/// This is what RuboCop's ParallelAssignment flags: only when every RHS
/// token is a plain literal, not a variable or method call.
fn is_simple_literal(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    // boolean / nil keywords
    if matches!(s, "true" | "false" | "nil") {
        return true;
    }
    // Symbol: :foo or :"foo"
    if s.starts_with(':') {
        return true;
    }
    // String literals: "..." or '...'
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
    {
        return true;
    }
    // Integer or float (optionally negative): -?[0-9][0-9_]*(\.[0-9]+)?
    let digits = s.strip_prefix('-').unwrap_or(s);
    if digits.starts_with(|c: char| c.is_ascii_digit()) {
        let rest = digits.trim_start_matches(|c: char| c.is_ascii_digit() || c == '_');
        if rest.is_empty() {
            return true;
        }
        // float
        if let Some(after_dot) = rest.strip_prefix('.') {
            if after_dot
                .chars()
                .all(|c| c.is_ascii_digit() || c == '_')
            {
                return true;
            }
        }
    }
    false
}

/// Split `s` on commas that are at nesting depth 0 (i.e., not inside
/// parentheses, brackets, or braces). Returns the individual tokens.
/// Returns `None` if there are no top-level commas (single expression).
fn split_top_level_commas(s: &str) -> Option<Vec<&str>> {
    let mut depth: i32 = 0;
    let mut last = 0usize;
    let mut tokens: Vec<&str> = Vec::new();
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
                tokens.push(&s[last..i]);
                last = i + 1;
            }
            _ => {}
        }
    }

    if tokens.is_empty() {
        // No top-level comma found
        return None;
    }

    tokens.push(&s[last..]);
    Some(tokens)
}

impl Rule for ParallelAssignment {
    fn name(&self) -> &'static str {
        "Style/ParallelAssignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comments
            if trimmed.starts_with('#') {
                continue;
            }

            // Skip method/class/module definitions — they use `=` for
            // default args and endless methods, both of which look like
            // parallel assignment to a naive parser.
            if trimmed.starts_with("def ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("module ")
            {
                continue;
            }

            // Find the first top-level ` = ` (not inside parens/brackets/braces)
            // We use a depth tracker to find the assignment at depth 0.
            let eq_pos = match find_top_level_eq(trimmed) {
                Some(p) => p,
                None => continue,
            };

            let lhs = &trimmed[..eq_pos];
            let rhs = trimmed[eq_pos + 3..].trim_end();

            // LHS must contain a comma (parallel variables)
            if !lhs.contains(',') {
                continue;
            }

            // LHS must not contain `(` or `)` — parenthesized LHS is
            // destructuring from a single call, not parallel assignment.
            // RuboCop does not flag those.
            if lhs.contains('(') || lhs.contains(')') {
                continue;
            }

            // LHS must not contain `*` (splat) — that signals array
            // destructuring, not parallel assignment.
            if lhs.contains('*') {
                continue;
            }

            // RHS: split on top-level commas.
            // If there are no top-level commas, the RHS is a single expression
            // (method call, variable, array literal) — destructuring, not parallel.
            let rhs_tokens = match split_top_level_commas(rhs) {
                Some(tokens) => tokens,
                None => continue,
            };

            // Every RHS token must be a simple literal. If any token is a
            // variable, method call, or compound value, this is destructuring —
            // not parallel assignment of literals.
            if !rhs_tokens.iter().all(|t| is_simple_literal(t)) {
                continue;
            }

            // Skip swap pattern `a, b = b, a`
            let lhs_tokens: Vec<&str> = lhs.split(',').map(|s| s.trim()).collect();
            if lhs_tokens.len() == rhs_tokens.len() {
                let reversed: Vec<&str> =
                    lhs_tokens.iter().copied().rev().collect();
                let rhs_trimmed: Vec<&str> =
                    rhs_tokens.iter().map(|s| s.trim()).collect();
                if reversed == rhs_trimmed {
                    continue;
                }
            }

            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let pos = (line_start + indent) as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Use sequential assignment instead of parallel assignment.".into(),
                range: TextRange::new(pos, pos + trimmed.len() as u32),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

/// Find the byte offset of a ` = ` sequence that is at nesting depth 0
/// (not inside parentheses, brackets, or braces).
fn find_top_level_eq(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth: i32 = 0;
    let len = bytes.len();

    let mut i = 0usize;
    while i < len {
        match bytes[i] {
            b'(' | b'[' | b'{' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' | b'}' => {
                if depth > 0 {
                    depth -= 1;
                }
                i += 1;
            }
            b' ' if depth == 0 && i + 2 < len && bytes[i + 1] == b'=' && bytes[i + 2] == b' ' => {
                // Make sure it's not `==`, `!=`, `<=`, `>=`
                let prev_ok = i == 0
                    || !matches!(bytes[i - 1], b'!' | b'<' | b'>' | b'=');
                // Make sure the char after ` = ` is not `=` (i.e., not ` == ` shifted)
                let next_ok = i + 3 >= len || bytes[i + 3] != b'=';
                if prev_ok && next_ok {
                    return Some(i);
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    None
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ConstantName;

/// Returns true if the name contains lowercase letters (meaning it's not SCREAMING_SNAKE_CASE).
fn has_lowercase(name: &str) -> bool {
    name.chars().any(|c| c.is_ascii_lowercase())
}

/// Returns true if the name looks like a class/module constant (CamelCase):
/// starts with uppercase, contains at least one lowercase letter, and has no underscores between
/// uppercase runs (e.g. `FooBar`, `MyClass`).
/// We consider it a class name if it has no underscores OR only underscores before non-uppercase runs.
fn is_class_style_constant(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    // Must start with uppercase
    if !name.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
        return false;
    }
    // If it has no underscores and has at least one lowercase, it's CamelCase → class style
    if !name.contains('_') && has_lowercase(name) {
        return true;
    }
    // If it has underscores but ALSO has lowercase, it's something like `Foo_Bar` → flag
    false
}

impl Rule for ConstantName {
    fn name(&self) -> &'static str {
        "Naming/ConstantName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                continue;
            }

            // Skip `class` and `module` definitions
            if trimmed.starts_with("class ") || trimmed.starts_with("module ") {
                continue;
            }

            // Look for a constant assignment: starts with uppercase letter
            let first_char = trimmed.chars().next().unwrap_or(' ');
            if !first_char.is_ascii_uppercase() {
                continue;
            }

            // Find `=` sign that is a simple assignment (not `==`, `=>`, `+=`, etc.)
            let eq_pos = if let Some(pos) = find_assignment_eq(trimmed) {
                pos
            } else {
                continue;
            };

            let const_name = trimmed[..eq_pos].trim_end();

            // Must look like an identifier (no spaces, parens etc.)
            if const_name
                .chars()
                .any(|c| c == '(' || c == ')' || c == '[' || c == ']' || c == ' ' || c == '.')
            {
                continue;
            }

            // Skip if it's a class-style constant (CamelCase with no underscores)
            if is_class_style_constant(const_name) {
                continue;
            }

            // Flag if the name has any lowercase letter
            if has_lowercase(const_name) {
                let line_start = ctx.line_start_offsets[i] as usize;
                let col = line.len() - trimmed.len();
                let start = (line_start + col) as u32;
                let end = (line_start + col + const_name.len()) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "Use SCREAMING_SNAKE_CASE for constants (`{}`).",
                        const_name
                    ),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Find position of a simple `=` that is an assignment (not `==`, `!=`, `<=`, `>=`, `=>`, `+=`, etc.)
fn find_assignment_eq(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        if bytes[i] == b'=' {
            // Check previous byte
            let prev_ok = i == 0
                || !matches!(
                    bytes[i - 1],
                    b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'^' | b'&'
                        | b'|' | b'~' | b'='
                );
            // Check next byte
            let next_ok = i + 1 >= n || (bytes[i + 1] != b'=' && bytes[i + 1] != b'>');
            if prev_ok && next_ok {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

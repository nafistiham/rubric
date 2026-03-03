use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MutableConstant;

impl Rule for MutableConstant {
    fn name(&self) -> &'static str {
        "Style/MutableConstant"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Match: CONSTANT_NAME = [ or { or "
            // Constant starts with uppercase letter
            let bytes = trimmed.as_bytes();
            if bytes.is_empty() || !bytes[0].is_ascii_uppercase() {
                continue;
            }

            // Find `=`
            let Some(eq_pos) = trimmed.find(" = ") else { continue; };

            // The LHS must be a bare constant name — no receiver before a dot.
            // `Devise.mailer = ...` is a setter method call, not a constant
            // assignment.  A Ruby constant is an unqualified uppercase
            // identifier, e.g. `FOO`, `MY_CONST`.
            let lhs = trimmed[..eq_pos].trim_end();
            if !is_bare_constant(lhs) {
                continue;
            }

            let rhs = trimmed[eq_pos + 3..].trim_start();

            // Check if rhs is a mutable literal (array, hash, string) without .freeze
            let is_mutable = rhs.starts_with('[') || rhs.starts_with('{') || rhs.starts_with('"') || rhs.starts_with('\'');
            let is_frozen = rhs.contains(".freeze");

            if is_mutable && !is_frozen {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                // Point to the start of the constant name
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Mutable object assigned to constant. Use `.freeze` to make it immutable.".into(),
                    range: TextRange::new(
                        (line_start + indent) as u32,
                        (line_start + indent + eq_pos) as u32,
                    ),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Returns true iff `s` is a bare Ruby constant name: starts with an
/// uppercase letter, contains only uppercase letters, digits, and underscores,
/// and has no `.` (which would indicate a receiver or namespace).
fn is_bare_constant(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Any dot means this is either a method call or a scoped constant access
    // with a receiver — not a bare local constant assignment.
    if s.contains('.') {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_uppercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

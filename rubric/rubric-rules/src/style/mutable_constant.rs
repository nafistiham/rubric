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

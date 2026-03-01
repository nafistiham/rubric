use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RaiseArgs;

impl Rule for RaiseArgs {
    fn name(&self) -> &'static str {
        "Style/RaiseArgs"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `raise ExceptionClass, "message"` pattern
            // Pattern: `raise ` followed by an uppercase word, then `, `
            if !trimmed.starts_with("raise ") {
                continue;
            }

            let after_raise = &trimmed[6..]; // skip "raise "
            let bytes = after_raise.as_bytes();

            // Check if it starts with uppercase (exception class)
            if bytes.is_empty() || !bytes[0].is_ascii_uppercase() {
                continue;
            }

            // Read the exception class name
            let mut j = 0;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b':') {
                j += 1;
            }

            // Check for `, ` after the class name
            if j < bytes.len() && bytes[j] == b',' {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let raise_start = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `raise ExceptionClass.new(msg)` instead of `raise ExceptionClass, msg`.".into(),
                    range: TextRange::new(raise_start, raise_start + 5),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

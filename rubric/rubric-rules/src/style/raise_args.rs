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

            // Only look at lines that start with `raise `
            if !trimmed.starts_with("raise ") {
                continue;
            }

            let after_raise = &trimmed[6..]; // skip "raise "
            let bytes = after_raise.as_bytes();

            // Must start with an uppercase letter (exception class, not variable/string)
            if bytes.is_empty() || !bytes[0].is_ascii_uppercase() {
                continue;
            }

            // Read the exception class name (allowing :: for namespaced constants)
            let mut j = 0;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b':')
            {
                j += 1;
            }

            // Check for `.new(` immediately after the class name
            let rest = &after_raise[j..];
            if !rest.starts_with(".new(") {
                continue;
            }

            // Skip `raise ExceptionClass.new()` — empty parens, no message to rewrite
            let inside = &rest[5..]; // skip ".new("
            if inside.starts_with(')') {
                continue;
            }

            // Flag: `raise ExceptionClass.new(msg)` — EnforcedStyle: exploded prefers comma form
            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let raise_start = (line_start + indent) as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Use `raise ExceptionClass, msg` instead of `raise ExceptionClass.new(msg)`.".into(),
                range: TextRange::new(raise_start, raise_start + 5),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

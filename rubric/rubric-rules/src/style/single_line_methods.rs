use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SingleLineMethods;

impl Rule for SingleLineMethods {
    fn name(&self) -> &'static str {
        "Style/SingleLineMethods"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("def ") {
                continue;
            }

            // A single-line method has `end` on the same line
            if !trimmed.ends_with(" end") && !trimmed.ends_with(";end") {
                continue;
            }

            // Exempt empty methods: `def foo; end`
            // Count the number of semicolons — if there's only the one before `end`, body is empty
            let without_end = &trimmed[..trimmed.len() - 4]; // strip " end"
            // If trimmed is "def foo; end", without_end is "def foo;"
            // Strip trailing semicolon and whitespace from without_end
            let without_end_trimmed = without_end.trim_end_matches(|c| c == ';' || c == ' ');
            // Check for additional semicolons (body content)
            if without_end_trimmed.contains(';') {
                // Has body content
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Avoid single-line method definitions with a body.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

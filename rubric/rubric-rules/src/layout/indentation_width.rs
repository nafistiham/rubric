use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IndentationWidth;

impl Rule for IndentationWidth {
    fn name(&self) -> &'static str {
        "Layout/IndentationWidth"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut prev_nonempty_line: &str = "";
        // Track when we're inside an inline conditional (x = if / x = unless / x = case)
        // whose continuation lines have alignment-based indentation.
        let mut inline_cond_depth: usize = 0;

        for (i, line) in ctx.lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            if line.starts_with('\t') {
                let start = ctx.line_start_offsets[i];
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use spaces, not tabs, for indentation.".into(),
                    range: TextRange::new(start, start + 1),
                    severity: Severity::Warning,
                });
                prev_nonempty_line = line;
                continue;
            }

            // Update inline conditional depth tracking
            if line.contains(" = if ") || line.contains(" = unless ") || line.contains(" = case ") {
                inline_cond_depth += 1;
            }

            let spaces = line.len() - line.trim_start_matches(' ').len();
            if spaces > 0 && spaces % 2 != 0 {
                // Skip continuation lines — trailing comma means aligned argument continuation.
                // Also skip lines inside inline conditional expressions (alignment to `if` keyword).
                let is_comma_continuation = prev_nonempty_line.trim_end().ends_with(',');
                if !is_comma_continuation && inline_cond_depth == 0 {
                    let start = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Indentation width must be a multiple of 2 (got {spaces})."),
                        range: TextRange::new(start, start + spaces as u32),
                        severity: Severity::Warning,
                    });
                }
            }

            // Decrement depth when we hit the `end` closing the inline conditional
            let trimmed = line.trim();
            if inline_cond_depth > 0 && (trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end.")) {
                inline_cond_depth -= 1;
            }

            prev_nonempty_line = line;
        }
        diags
    }
}

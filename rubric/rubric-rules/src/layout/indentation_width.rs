use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IndentationWidth;

impl Rule for IndentationWidth {
    fn name(&self) -> &'static str {
        "Layout/IndentationWidth"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut prev_nonempty_line: &str = ""; // track previous non-empty line to detect continuations
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
            let spaces = line.len() - line.trim_start_matches(' ').len();
            if spaces > 0 && spaces % 2 != 0 {
                // Skip continuation lines — a trailing comma on the previous non-empty
                // line means this line is an aligned argument continuation, not a new
                // block scope indent.
                if !prev_nonempty_line.trim_end().ends_with(',') {
                    let start = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Indentation width must be a multiple of 2 (got {spaces})."),
                        range: TextRange::new(start, start + spaces as u32),
                        severity: Severity::Warning,
                    });
                }
            }
            prev_nonempty_line = line;
        }
        diags
    }
}

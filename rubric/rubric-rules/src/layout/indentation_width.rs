use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IndentationWidth;

impl Rule for IndentationWidth {
    fn name(&self) -> &'static str {
        "Layout/IndentationWidth"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
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
                continue;
            }
            let spaces = line.len() - line.trim_start_matches(' ').len();
            if spaces > 0 && spaces % 2 != 0 {
                let start = ctx.line_start_offsets[i];
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!("Indentation width must be a multiple of 2 (got {spaces})."),
                    range: TextRange::new(start, start + spaces as u32),
                    severity: Severity::Warning,
                });
            }
        }
        diags
    }
}

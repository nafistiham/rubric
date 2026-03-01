use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineMethodCallIndentation;

impl Rule for MultilineMethodCallIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodCallIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_end();
            // A line ending with `.` indicates a chained call continuation (trailing dot style)
            if trimmed.ends_with('.') {
                let line_start = ctx.line_start_offsets[i] as usize;
                let dot_offset = line_start + trimmed.len() - 1;
                let pos = dot_offset as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Chained method call continuation detected — ensure proper indentation.".into(),
                    range: TextRange::new(pos, pos + 1),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

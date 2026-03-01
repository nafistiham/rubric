use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct IndentationStyle;

impl Rule for IndentationStyle {
    fn name(&self) -> &'static str {
        "Layout/IndentationStyle"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            // Count leading tabs
            let tab_count = line.bytes().take_while(|&b| b == b'\t').count();
            if tab_count > 0 {
                let line_start = ctx.line_start_offsets[i] as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use spaces for indentation instead of tabs.".into(),
                    range: TextRange::new(line_start, line_start + tab_count as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // Replace each tab with 2 spaces — but we don't know the count from just the range.
        // Return None here since we'd need the source to count tabs properly.
        let _ = diag;
        None
    }
}

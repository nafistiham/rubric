use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineMethodCallBraceLayout;

impl Rule for MultilineMethodCallBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodCallBraceLayout"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 1..n {
            let line = &lines[i];
            let trimmed = line.trim();
            // Closing `)` on same line as last argument in multiline call
            if trimmed.ends_with(')') && !trimmed.starts_with(')') && trimmed.len() > 1 {
                // Check if previous lines have unclosed `(`
                let mut j = i;
                let mut is_multiline = false;
                while j > 0 {
                    j -= 1;
                    let prev = lines[j].trim();
                    if prev.contains('(') {
                        is_multiline = true;
                        break;
                    }
                    if prev.is_empty() { break; }
                }

                if is_multiline {
                    let indent = line.len() - line.trim_start().len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Closing `)` of multiline method call should be on its own line.".into(),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ClosingParenthesisIndentation;

impl Rule for ClosingParenthesisIndentation {
    fn name(&self) -> &'static str {
        "Layout/ClosingParenthesisIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_end();
            // A lone `)` on its own line
            if trimmed.trim_start() != ")" {
                continue;
            }
            // Count leading spaces
            let indent = trimmed.len() - trimmed.trim_start().len();
            // Flag if indentation is not a multiple of 2
            if indent % 2 != 0 {
                let line_start = ctx.line_start_offsets[i] as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "Closing `)` has odd indentation ({} spaces); expected a multiple of 2.",
                        indent
                    ),
                    range: TextRange::new(line_start, line_start + indent as u32 + 1),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

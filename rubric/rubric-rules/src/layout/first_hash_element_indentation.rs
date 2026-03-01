use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FirstHashElementIndentation;

impl Rule for FirstHashElementIndentation {
    fn name(&self) -> &'static str {
        "Layout/FirstHashElementIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_end();

            // Check if this line ends with `{` (indicating a multi-line hash opening)
            if !trimmed.ends_with('{') {
                continue;
            }

            // Compute the opener's indentation
            let opener_indent = trimmed.len() - trimmed.trim_start().len();

            // Find the next non-empty line
            let mut next_idx = i + 1;
            while next_idx < n && lines[next_idx].trim().is_empty() {
                next_idx += 1;
            }

            if next_idx >= n {
                continue;
            }

            let next_line = &lines[next_idx];
            let next_trimmed = next_line.trim_start();

            // Skip closing brace (empty hash)
            if next_trimmed.starts_with('}') {
                continue;
            }

            let next_indent = next_line.len() - next_trimmed.len();
            let expected_indent = opener_indent + 2;

            if next_indent != expected_indent {
                let next_line_start = ctx.line_start_offsets[next_idx] as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "First hash element should be indented by {} spaces (got {}).",
                        expected_indent, next_indent
                    ),
                    range: TextRange::new(next_line_start, next_line_start + next_indent as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

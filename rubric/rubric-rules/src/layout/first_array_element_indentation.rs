use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FirstArrayElementIndentation;

impl Rule for FirstArrayElementIndentation {
    fn name(&self) -> &'static str {
        "Layout/FirstArrayElementIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect line ending with `[` (multiline array start)
            if trimmed.ends_with('[') && i + 1 < n {
                let line_indent = line.len() - trimmed.len();
                // Consistent style: line_indent + 2
                let expected_consistent = line_indent + 2;
                // Special-inside-parentheses style: align to bracket column + 1 or + 2
                // bracket column = line_indent + position of `[` within trimmed
                let bracket_col = line_indent + trimmed.len() - 1;
                let expected_aligned_1 = bracket_col + 1;
                let expected_aligned_2 = bracket_col + 2;

                let next_line = &lines[i + 1];
                let actual_indent = next_line.len() - next_line.trim_start().len();

                if !next_line.trim().is_empty()
                    && actual_indent != expected_consistent
                    && actual_indent != expected_aligned_1
                    && actual_indent != expected_aligned_2
                {
                    let line_start = ctx.line_start_offsets[i + 1] as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "First array element should be indented {} spaces, not {}.",
                            expected_consistent, actual_indent
                        ),
                        range: TextRange::new(line_start, line_start + actual_indent as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

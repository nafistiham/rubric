use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FirstParameterIndentation;

impl Rule for FirstParameterIndentation {
    fn name(&self) -> &'static str {
        "Layout/FirstParameterIndentation"
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

            // Detect `def foo(` ending with `(` (multiline def start)
            if trimmed.starts_with("def ") && trimmed.ends_with('(') && i + 1 < n {
                let def_indent = line.len() - trimmed.len();
                let expected_indent = def_indent + 2;
                let next_line = &lines[i + 1];
                let actual_indent = next_line.len() - next_line.trim_start().len();

                if !next_line.trim().is_empty() && actual_indent != expected_indent {
                    let line_start = ctx.line_start_offsets[i + 1] as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "First parameter should be indented {} spaces, not {}.",
                            expected_indent, actual_indent
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

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLinesAroundMethodBody;

impl Rule for EmptyLinesAroundMethodBody {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundMethodBody"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Stack: true = def opener, false = other opener
        let mut opener_stack: Vec<bool> = Vec::new();

        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();

            if trimmed.starts_with("def ") || trimmed == "def" {
                // Check if the next line is blank
                if i + 1 < n && lines[i + 1].trim().is_empty() {
                    let line_start = ctx.line_start_offsets[i + 1];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra empty line after method body start.".into(),
                        range: TextRange::new(line_start, line_start + lines[i + 1].len() as u32),
                        severity: Severity::Warning,
                    });
                }
                opener_stack.push(true);
            } else if trimmed.starts_with("class ") || trimmed.starts_with("module ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed.starts_with("begin")
                || trimmed == "do" || trimmed.ends_with(" do") {
                opener_stack.push(false);
            } else if trimmed == "end" {
                if let Some(is_def) = opener_stack.pop() {
                    if is_def {
                        // Check if line before `end` is blank
                        if i > 0 && lines[i - 1].trim().is_empty() {
                            let line_start = ctx.line_start_offsets[i - 1];
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Extra empty line before method body end.".into(),
                                range: TextRange::new(
                                    line_start,
                                    line_start + lines[i - 1].len() as u32,
                                ),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
            i += 1;
        }

        diags
    }
}

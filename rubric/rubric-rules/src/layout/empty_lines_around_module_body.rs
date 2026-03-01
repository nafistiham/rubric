use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLinesAroundModuleBody;

impl Rule for EmptyLinesAroundModuleBody {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundModuleBody"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track module depth to find module-closing `end` lines
        // Stack stores: true = module opener, false = other opener
        let mut opener_stack: Vec<bool> = Vec::new();

        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();

            if trimmed.starts_with("module ") || trimmed == "module" {
                // Check if the next line is blank
                if i + 1 < n && lines[i + 1].trim().is_empty() {
                    let line_start = ctx.line_start_offsets[i + 1];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra empty line after module body start.".into(),
                        range: TextRange::new(line_start, line_start + lines[i + 1].len() as u32),
                        severity: Severity::Warning,
                    });
                }
                opener_stack.push(true);
            } else if trimmed.starts_with("class ") || trimmed.starts_with("def ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed.starts_with("begin")
                || trimmed == "do" || trimmed.ends_with(" do") {
                opener_stack.push(false);
            } else if trimmed == "end" {
                if let Some(is_module) = opener_stack.pop() {
                    if is_module {
                        // Check if the line before this `end` is blank
                        if i > 0 && lines[i - 1].trim().is_empty() {
                            let line_start = ctx.line_start_offsets[i - 1];
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Extra empty line before module body end.".into(),
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

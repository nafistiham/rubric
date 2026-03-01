use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLinesAroundClassBody;

impl Rule for EmptyLinesAroundClassBody {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundClassBody"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track class depth to find class-closing `end` lines
        let mut class_open_lines: Vec<usize> = Vec::new();

        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();

            if trimmed.starts_with("class ") || trimmed == "class" {
                // Check if the next line is blank (empty line after class declaration)
                if i + 1 < n && lines[i + 1].trim().is_empty() {
                    let line_start = ctx.line_start_offsets[i + 1];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra empty line after class body start.".into(),
                        range: TextRange::new(line_start, line_start + lines[i + 1].len() as u32),
                        severity: Severity::Warning,
                    });
                }
                class_open_lines.push(i);
            } else if trimmed.starts_with("module ") || trimmed.starts_with("def ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed.starts_with("begin")
                || trimmed == "do" || trimmed.ends_with(" do") {
                // Non-class openers that add depth — push a sentinel (usize::MAX)
                class_open_lines.push(usize::MAX);
            } else if trimmed == "end" {
                if let Some(opener) = class_open_lines.pop() {
                    // If this `end` closes a class, check if the line before it is blank
                    if opener != usize::MAX {
                        // opener is a class line — check if line before this `end` is blank
                        if i > 0 && lines[i - 1].trim().is_empty() {
                            let line_start = ctx.line_start_offsets[i - 1];
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Extra empty line before class body end.".into(),
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

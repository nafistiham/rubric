use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantReturn;

impl Rule for RedundantReturn {
    fn name(&self) -> &'static str {
        "Style/RedundantReturn"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Simple state machine: track def/end blocks
        let mut def_depth = 0i32;
        let _in_def = false;

        // We need to find the last content line before each `end` that closes a `def`
        // Track def-start positions to know when we're at depth 1
        let mut def_depth_stack: Vec<i32> = Vec::new();
        // We'll track "last content line before end" using a two-pass approach

        // Actually: simplest approach — track all `def..end` blocks linearly
        // When depth drops from 1->0 (a `def` end), examine the last content line before that `end`

        let mut last_content_line: Option<usize> = None;
        let mut current_def_start: Option<usize> = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("def ") || trimmed == "def" {
                def_depth += 1;
                def_depth_stack.push(def_depth);
                if def_depth == 1 {
                    current_def_start = Some(i);
                    last_content_line = None;
                }
            } else if trimmed == "end" && def_depth > 0 {
                if def_depth == *def_depth_stack.last().unwrap_or(&0) {
                    // This `end` closes the tracked def
                    if def_depth == 1 {
                        // Check last content line
                        if let Some(last_idx) = last_content_line {
                            let last_line = lines[last_idx].trim_start();
                            if last_line.starts_with("return ") || last_line == "return" {
                                let last_line_start = ctx.line_start_offsets[last_idx] as usize;
                                let indent = lines[last_idx].len() - lines[last_idx].trim_start().len();
                                let pos = (last_line_start + indent) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Redundant `return` in last statement of method.".into(),
                                    range: TextRange::new(pos, pos + 6),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                        current_def_start = None;
                        last_content_line = None;
                    }
                    def_depth_stack.pop();
                }
                def_depth -= 1;
            } else if def_depth == 1 && !trimmed.is_empty() && !trimmed.starts_with('#') {
                last_content_line = Some(i);
            }
        }

        let _ = current_def_start;
        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IndentationConsistency;

impl Rule for IndentationConsistency {
    fn name(&self) -> &'static str {
        "Layout/IndentationConsistency"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            // Check if the leading whitespace mixes tabs and spaces
            let bytes = line.as_bytes();
            let mut has_tab = false;
            let mut has_space_after_tab = false;
            let mut j = 0;
            let mut seen_tab = false;

            while j < bytes.len() {
                match bytes[j] {
                    b'\t' => {
                        has_tab = true;
                        if j > 0 && bytes[j - 1] == b' ' {
                            // space before tab
                            has_space_after_tab = true;
                        }
                        seen_tab = true;
                    }
                    b' ' => {
                        if seen_tab {
                            // space after tab in leading whitespace
                            has_space_after_tab = true;
                        }
                    }
                    _ => break,
                }
                j += 1;
            }

            if has_tab && has_space_after_tab {
                let line_start = ctx.line_start_offsets[i];
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Mixed tabs and spaces in indentation.".into(),
                    range: TextRange::new(line_start, line_start + j as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct EmptyMethod;

impl Rule for EmptyMethod {
    fn name(&self) -> &'static str {
        "Style/EmptyMethod"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let trimmed = lines[i].trim_start();
            if trimmed.starts_with("def ") || trimmed == "def" {
                // Check if the very next non-empty line is `end`
                if i + 1 < n {
                    let next = lines[i + 1].trim();
                    if next == "end" {
                        let indent = lines[i].len() - trimmed.len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        // Range covers from def to end of `end` line
                        let end_line_start = ctx.line_start_offsets[i + 1] as usize;
                        let end_line = &lines[i + 1];
                        let end_pos = (end_line_start + end_line.len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Empty method body; use `def foo; end` on a single line.".into(),
                            range: TextRange::new(pos, end_pos),
                            severity: Severity::Warning,
                        });
                        i += 2;
                        continue;
                    }
                }
            }
            i += 1;
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // We cannot easily reconstruct the def signature here from just the range,
        // so we return None (fix requires source context beyond what Diagnostic carries).
        // The fix is handled by the check_source logic above for reporting only.
        let _ = diag;
        None
    }
}

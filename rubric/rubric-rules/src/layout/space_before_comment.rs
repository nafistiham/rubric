use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceBeforeComment;

impl Rule for SpaceBeforeComment {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeComment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;
            for (j, &b) in bytes.iter().enumerate() {
                if b == b'#' && j > 0 {
                    let prev = bytes[j - 1];
                    if prev != b' ' && prev != b'\t' {
                        let pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Put a space before an inline comment.".into(),
                            range: TextRange::new(pos, pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                    break; // Only one `#` per line matters (first one)
                }
            }
        }
        diags
    }
}

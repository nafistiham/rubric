use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct EndOfLine;

impl Rule for EndOfLine {
    fn name(&self) -> &'static str {
        "Layout/EndOfLine"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;
        let bytes = src.as_bytes();
        let n = bytes.len();

        let mut i = 0;
        while i < n {
            if bytes[i] == b'\r' && i + 1 < n && bytes[i + 1] == b'\n' {
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Windows-style (CRLF) line endings detected; use LF only.".into(),
                    range: TextRange::new(i as u32, (i + 1) as u32),
                    severity: Severity::Warning,
                });
            }
            i += 1;
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: String::new(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

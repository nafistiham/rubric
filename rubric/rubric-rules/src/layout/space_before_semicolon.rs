use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceBeforeSemicolon;

impl Rule for SpaceBeforeSemicolon {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeSemicolon"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < n {
                let b = bytes[j];
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break,
                    None => {}
                }

                // Check for space before semicolon: byte at j is space, byte at j+1 is `;`
                if b == b' ' && j + 1 < n && bytes[j + 1] == b';' {
                    let abs_pos = line_start + j;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Avoid space before semicolon.".into(),
                        range: TextRange::new(abs_pos as u32, (abs_pos + 1) as u32),
                        severity: Severity::Warning,
                    });
                }

                j += 1;
            }
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

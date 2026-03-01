use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceAfterComma;

impl Rule for SpaceAfterComma {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterComma"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let line_bytes = line.as_bytes();
            let mut in_string: Option<u8> = None; // None = outside, Some(b'"') or Some(b'\'') = inside
            let mut j = 0;
            while j < line_bytes.len() {
                let b = line_bytes[j];
                match in_string {
                    // Inside a string: handle escape sequences and closing delimiter
                    Some(_) if b == b'\\' => { j += 2; continue; } // skip escaped char
                    Some(delim) if b == delim => { in_string = None; }
                    Some(_) => {}
                    // Outside a string
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); }
                    None if b == b',' => {
                        let next = line_bytes.get(j + 1).copied();
                        if next != Some(b' ') && next != Some(b'\t') && next.is_some() {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space missing after comma.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    None => {}
                }
                j += 1;
            }
        }
        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: TextRange::new(diag.range.start, diag.range.end),
                replacement: ", ".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

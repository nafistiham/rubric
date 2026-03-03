use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceAroundBlockParameters;

impl Rule for SpaceAroundBlockParameters {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundBlockParameters"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();

            let mut j = 0;
            let mut in_string: Option<u8> = None;
            while j < len {
                let b = bytes[j];
                // String tracking — skip characters inside string literals
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment
                    None => {}
                }

                // Detect `{|` — open brace immediately followed by pipe (no space)
                if bytes[j] == b'{' && j + 1 < len && bytes[j+1] == b'|' {
                    let pos = (line_start + j) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Missing space between `{` and `|` in block parameters.".into(),
                        range: TextRange::new(pos, pos + 2),
                        severity: Severity::Warning,
                    });
                }

                // Detect `|}` — pipe immediately before close brace (no space)
                if bytes[j] == b'|' && j + 1 < len && bytes[j+1] == b'}' {
                    let pos = (line_start + j) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Missing space between `|` and `}` in block parameters.".into(),
                        range: TextRange::new(pos, pos + 2),
                        severity: Severity::Warning,
                    });
                }

                j += 1;
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // The diagnostic range always spans 2 bytes: either `{|` or `|}`
        // Distinguish by the message content
        if diag.message.contains("between `{` and `|`") {
            Some(Fix {
                edits: vec![TextEdit {
                    range: TextRange::new(diag.range.start, diag.range.start + 2),
                    replacement: "{ |".into(),
                }],
                safety: FixSafety::Safe,
            })
        } else {
            Some(Fix {
                edits: vec![TextEdit {
                    range: TextRange::new(diag.range.start, diag.range.start + 2),
                    replacement: "| }".into(),
                }],
                safety: FixSafety::Safe,
            })
        }
    }
}

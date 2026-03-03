use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceAfterComma;

impl Rule for SpaceAfterComma {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterComma"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip pure comment lines
            if line.trim_start().starts_with('#') {
                continue;
            }
            let line_start = ctx.line_start_offsets[i] as usize;
            let line_bytes = line.as_bytes();
            let mut in_string: Option<u8> = None; // None = outside, Some(delim) = inside string
            let mut j = 0;
            while j < line_bytes.len() {
                let b = line_bytes[j];
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; } // skip escaped char
                    Some(b'"') if b == b'#' && j + 1 < line_bytes.len() && line_bytes[j + 1] == b'{' => {
                        // String interpolation: skip #{...} block, tracking nested braces and strings
                        j += 2; // skip #{
                        let mut depth = 1usize;
                        while j < line_bytes.len() && depth > 0 {
                            let ib = line_bytes[j];
                            if ib == b'\\' { j += 2; continue; }
                            if ib == b'{' { depth += 1; j += 1; continue; }
                            if ib == b'}' {
                                depth -= 1;
                                if depth == 0 { j += 1; break; }
                                j += 1;
                                continue;
                            }
                            // Nested string inside interpolation — skip its content
                            if ib == b'"' || ib == b'\'' || ib == b'`' {
                                let id = ib;
                                j += 1;
                                while j < line_bytes.len() {
                                    if line_bytes[j] == b'\\' { j += 2; continue; }
                                    if line_bytes[j] == id { j += 1; break; }
                                    j += 1;
                                }
                                continue;
                            }
                            j += 1;
                        }
                        continue;
                    }
                    Some(delim) if b == delim => { in_string = None; }
                    Some(_) => {}
                    // Outside a string
                    None if b == b'"' || b == b'\'' || b == b'`' => { in_string = Some(b); }
                    None if b == b'#' => break, // inline comment — stop scanning
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

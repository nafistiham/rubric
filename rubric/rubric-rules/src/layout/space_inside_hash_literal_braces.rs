use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceInsideHashLiteralBraces;

impl Rule for SpaceInsideHashLiteralBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideHashLiteralBraces"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;

            let mut j = 0;
            while j < len {
                let b = bytes[j];

                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break,
                    None => {}
                }

                // Skip %r{...} and other %r delimiters — regex literals
                if b == b'%' && j + 1 < len && bytes[j + 1] == b'r' {
                    j += 2;
                    if j < len {
                        let delim = bytes[j];
                        j += 1;
                        if delim == b'{' {
                            let mut depth = 1usize;
                            while j < len && depth > 0 {
                                match bytes[j] {
                                    b'\\' => { j += 2; }
                                    b'{' => { depth += 1; j += 1; }
                                    b'}' => { depth -= 1; j += 1; }
                                    _ => { j += 1; }
                                }
                            }
                        } else {
                            while j < len && bytes[j] != delim {
                                if bytes[j] == b'\\' { j += 2; } else { j += 1; }
                            }
                            if j < len { j += 1; }
                        }
                    }
                    continue;
                }

                // Skip /regex/ literals
                if b == b'/' {
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    if prev == b'=' || prev == b'(' || prev == b','
                        || prev == b'[' || prev == b' ' || prev == b'\t' || prev == 0
                    {
                        j += 1;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'/' => { j += 1; break; }
                                _ => { j += 1; }
                            }
                        }
                        continue;
                    }
                }

                // Detect `{` not followed by space and not empty `{}`
                if b == b'{' {
                    let next = if j + 1 < len { bytes[j+1] } else { 0 };
                    // Skip empty braces `{}`
                    if next == b'}' {
                        j += 2;
                        continue;
                    }
                    // Flag if next char is not a space
                    if next != b' ' && next != b'\n' && next != 0 {
                        let pos = (line_start + j + 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Missing space after `{` in hash literal.".into(),
                            range: TextRange::new(pos, pos),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Detect `}` not preceded by space and not empty `{}`
                if b == b'}' {
                    let prev = if j > 0 { bytes[j-1] } else { 0 };
                    // Skip empty braces already handled above
                    if prev == b'{' {
                        j += 1;
                        continue;
                    }
                    // Flag if prev char is not a space
                    if prev != b' ' && prev != 0 {
                        let pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Missing space before `}` in hash literal.".into(),
                            range: TextRange::new(pos, pos),
                            severity: Severity::Warning,
                        });
                    }
                }

                j += 1;
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // Insert a space at the flagged position
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: " ".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

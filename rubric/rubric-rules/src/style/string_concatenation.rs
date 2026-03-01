use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct StringConcatenation;

impl Rule for StringConcatenation {
    fn name(&self) -> &'static str {
        "Style/StringConcatenation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Look for `" +` or `' +` patterns (string literal ends, then ` +`)
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut pos = 0;
            let mut in_string: Option<u8> = None;

            while pos < n {
                let b = bytes[pos];
                match in_string {
                    Some(_) if b == b'\\' => { pos += 2; continue; }
                    Some(delim) if b == delim => {
                        // String just ended — check if followed by ` +`
                        in_string = None;
                        pos += 1;
                        // Skip whitespace
                        let mut j = pos;
                        while j < n && bytes[j] == b' ' { j += 1; }
                        if j < n && bytes[j] == b'+' {
                            // Check it's not `+=`
                            let after_plus = if j + 1 < n { bytes[j + 1] } else { 0 };
                            if after_plus != b'=' {
                                let line_start = ctx.line_start_offsets[i] as usize;
                                let flag_pos = (line_start + j) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use string interpolation instead of string concatenation.".into(),
                                    range: TextRange::new(flag_pos, flag_pos + 1),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                        continue;
                    }
                    Some(_) => { pos += 1; continue; }
                    None if b == b'"' || b == b'\'' => {
                        in_string = Some(b);
                        pos += 1;
                        continue;
                    }
                    None if b == b'#' => break,
                    None => {}
                }
                pos += 1;
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ImplicitStringConcatenation;

impl Rule for ImplicitStringConcatenation {
    fn name(&self) -> &'static str {
        "Lint/ImplicitStringConcatenation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut pos = 0;
            let mut in_string: Option<u8> = None;
            let mut string_ended_at: Option<usize> = None;

            while pos < n {
                let b = bytes[pos];
                match in_string {
                    Some(_) if b == b'\\' => {
                        string_ended_at = None;
                        pos += 2;
                        continue;
                    }
                    Some(delim) if b == delim => {
                        in_string = None;
                        string_ended_at = Some(pos);
                        pos += 1;
                        continue;
                    }
                    Some(_) => { pos += 1; continue; }
                    None if b == b'"' || b == b'\'' => {
                        // If we just ended a string and now starting a new one
                        if let Some(_end_pos) = string_ended_at {
                            // Check if there's only whitespace between them
                            let line_start = ctx.line_start_offsets[i] as usize;
                            let flag_pos = (line_start + pos) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Implicit string concatenation detected.".into(),
                                range: TextRange::new(flag_pos, flag_pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                        in_string = Some(b);
                        string_ended_at = None;
                        pos += 1;
                        continue;
                    }
                    None if b == b' ' => {
                        // whitespace between strings is ok
                        pos += 1;
                        continue;
                    }
                    None if b == b'#' => break,
                    None => {
                        string_ended_at = None;
                        pos += 1;
                        continue;
                    }
                }
            }
        }

        diags
    }
}

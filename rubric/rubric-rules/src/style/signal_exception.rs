use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SignalException;

impl Rule for SignalException {
    fn name(&self) -> &'static str {
        "Style/SignalException"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let len = bytes.len();
            let pattern = b"fail";
            let pat_len = pattern.len();
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

                if j + pat_len <= len && &bytes[j..j + pat_len] == pattern {
                    let before_ok = j == 0 || (!bytes[j - 1].is_ascii_alphanumeric() && bytes[j - 1] != b'_');
                    let after_pos = j + pat_len;
                    let after_ok = after_pos >= len || bytes[after_pos] == b' ' || bytes[after_pos] == b'\n';

                    if before_ok && after_ok {
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Use `raise` instead of `fail`.".into(),
                            range: TextRange::new(line_start + j as u32, line_start + (j + pat_len) as u32),
                            severity: Severity::Warning,
                        });
                        break;
                    }
                }
                j += 1;
            }
        }

        diags
    }
}

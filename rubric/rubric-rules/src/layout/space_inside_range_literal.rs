use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceInsideRangeLiteral;

impl Rule for SpaceInsideRangeLiteral {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideRangeLiteral"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

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

                // Check for ` .. ` or ` ... ` patterns
                for pattern in &[" ... ", " .. "] {
                    let pb = pattern.as_bytes();
                    let pn = pb.len();
                    if j + pn <= n && &bytes[j..j + pn] == pb {
                        let abs_pos = line_start + j;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Avoid spaces inside range literals.".into(),
                            range: TextRange::new(abs_pos as u32, (abs_pos + pn) as u32),
                            severity: Severity::Warning,
                        });
                        // Only flag once per position (prefer longer match)
                        break;
                    }
                }

                j += 1;
            }
        }

        diags
    }
}

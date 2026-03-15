use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HashConversion;

impl Rule for HashConversion {
    fn name(&self) -> &'static str {
        "Style/HashConversion"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Look for `Hash[` — the Hash class constructor.
            // Pattern: word boundary before `Hash`, then `[`.
            let mut search = 0;
            while search + 5 <= bytes.len() {
                if let Some(rel) = bytes[search..]
                    .windows(5)
                    .position(|w| w == b"Hash[")
                {
                    let abs = search + rel;

                    // Ensure word boundary before `Hash`
                    let before_ok = abs == 0
                        || {
                            let b = bytes[abs - 1];
                            !b.is_ascii_alphanumeric() && b != b'_'
                        };

                    // Ensure this is not inside a string or comment
                    let in_str = is_in_string_or_comment(bytes, abs);

                    if before_ok && !in_str {
                        let start = (line_start + abs) as u32;
                        let end = start + 5; // len("Hash[")
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Prefer array.to_h over Hash[array].".into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                    }

                    search = abs + 5;
                } else {
                    break;
                }
            }
        }

        diags
    }
}

/// Returns true if the byte at `pos` in `bytes` is inside a string literal or after a comment `#`.
fn is_in_string_or_comment(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos && i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => {
                // Real comment — pos is after comment start
                return true;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

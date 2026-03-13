use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct YodaCondition;

// Patterns: literal on left of comparison
// We match literal words followed by == or !=
const YODA_LITERALS: &[&str] = &["nil", "true", "false"];

/// Returns true if `pos` in `bytes` is inside a `"` or `'` string literal.
fn in_string_context(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos {
        match in_str {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(d) if bytes[i] == d => { in_str = None; }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => { in_str = Some(bytes[i]); }
            None if bytes[i] == b'#' => break,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

impl Rule for YodaCondition {
    fn name(&self) -> &'static str {
        "Style/YodaCondition"
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

            // Check for literal keywords followed by == or !=
            for literal in YODA_LITERALS {
                let lit_bytes = literal.as_bytes();
                let lit_len = lit_bytes.len();
                let mut j = 0;
                while j + lit_len <= len {
                    if &bytes[j..j + lit_len] == lit_bytes {
                        let before_ok = j == 0 || (!bytes[j - 1].is_ascii_alphanumeric() && bytes[j - 1] != b'_');
                        let after_pos = j + lit_len;
                        let after_ok = after_pos >= len || (!bytes[after_pos].is_ascii_alphanumeric() && bytes[after_pos] != b'_');

                        if before_ok && after_ok && after_pos + 1 < len {
                            // Check for ` ==` or ` !=` after literal
                            let rest = &bytes[after_pos..];
                            let is_yoda = (rest.len() >= 3 && rest[0] == b' ' && rest[1] == b'=' && rest[2] == b'=')
                                || (rest.len() >= 3 && rest[0] == b' ' && rest[1] == b'!' && rest[2] == b'=');

                            if is_yoda && !in_string_context(bytes, j) {
                                let line_start = ctx.line_start_offsets[i];
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!("Yoda condition: literal `{}` on left side of comparison.", literal),
                                    range: TextRange::new(line_start + j as u32, line_start + (j + lit_len) as u32),
                                    severity: Severity::Warning,
                                });
                                break;
                            }
                        }
                    }
                    j += 1;
                }
            }

            // Also detect numeric literals: `\d+ ==` or `\d+ !=`
            let mut j = 0;
            while j < len {
                if bytes[j].is_ascii_digit() {
                    let num_start = j;
                    while j < len && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
                        j += 1;
                    }
                    let num_end = j;
                    // Check word boundary before (not part of another number)
                    let before_ok = num_start == 0 || (!bytes[num_start - 1].is_ascii_alphanumeric() && bytes[num_start - 1] != b'_');

                    if before_ok && num_end < len && !in_string_context(bytes, num_start) {
                        let rest = &bytes[num_end..];
                        let is_yoda = (rest.len() >= 3 && rest[0] == b' ' && rest[1] == b'=' && rest[2] == b'=')
                            || (rest.len() >= 3 && rest[0] == b' ' && rest[1] == b'!' && rest[2] == b'=');

                        if is_yoda {
                            let line_start = ctx.line_start_offsets[i];
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Yoda condition: literal on left side of comparison.".into(),
                                range: TextRange::new(line_start + num_start as u32, line_start + num_end as u32),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    continue;
                }
                j += 1;
            }
        }

        diags
    }
}

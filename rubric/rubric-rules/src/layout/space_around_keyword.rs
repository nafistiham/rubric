use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceAroundKeyword;

// Keywords that must be followed by a space (not immediately by `(`)
const KEYWORDS: &[&str] = &["if", "unless", "while", "until", "and", "or", "not", "in", "do"];

impl Rule for SpaceAroundKeyword {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundKeyword"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let len = bytes.len();

            for kw in KEYWORDS {
                let kw_bytes = kw.as_bytes();
                let kw_len = kw_bytes.len();
                let mut j = 0;
                while j + kw_len <= len {
                    // Check if this position has the keyword
                    if &bytes[j..j + kw_len] == kw_bytes {
                        // Check word boundary before
                        let before_ok = j == 0 || !bytes[j - 1].is_ascii_alphanumeric() && bytes[j - 1] != b'_';
                        // Check what comes after the keyword
                        let after_pos = j + kw_len;
                        if before_ok && after_pos < len && bytes[after_pos] == b'(' {
                            // keyword immediately followed by '(' — violation
                            let line_start = ctx.line_start_offsets[i];
                            let kw_start = line_start + j as u32;
                            let kw_end = kw_start + kw_len as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Keyword `{}` should be followed by a space.",
                                    kw
                                ),
                                range: TextRange::new(kw_start, kw_end),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    j += 1;
                }
            }
        }

        diags
    }
}

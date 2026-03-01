use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct Lambda;

impl Rule for Lambda {
    fn name(&self) -> &'static str {
        "Style/Lambda"
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

            // Detect `lambda {` or `lambda do` patterns
            // Look for `lambda` followed by space and `{` or `do`
            let bytes = line.as_bytes();
            let len = bytes.len();
            let pattern = b"lambda";
            let pat_len = pattern.len();

            let mut j = 0;
            while j + pat_len <= len {
                if &bytes[j..j + pat_len] == pattern {
                    // Check word boundary before
                    let before_ok = j == 0 || (!bytes[j - 1].is_ascii_alphanumeric() && bytes[j - 1] != b'_');
                    // Check what comes after
                    let after_pos = j + pat_len;
                    let after_ok = after_pos >= len
                        || (!bytes[after_pos].is_ascii_alphanumeric() && bytes[after_pos] != b'_');

                    if before_ok && after_ok {
                        // Check for `lambda {` or `lambda do`
                        if after_pos < len && bytes[after_pos] == b' ' {
                            let next_non_space = after_pos + 1;
                            if next_non_space < len
                                && (bytes[next_non_space] == b'{'
                                    || (next_non_space + 1 < len
                                        && &bytes[next_non_space..next_non_space + 2] == b"do"))
                            {
                                let line_start = ctx.line_start_offsets[i];
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use `->` (stabby lambda) instead of `lambda`.".into(),
                                    range: TextRange::new(
                                        line_start + j as u32,
                                        line_start + (j + pat_len) as u32,
                                    ),
                                    severity: Severity::Warning,
                                });
                                break; // one violation per line
                            }
                        }
                    }
                }
                j += 1;
            }
        }

        diags
    }
}

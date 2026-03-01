use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashSet;

pub struct DuplicateHashKey;

impl Rule for DuplicateHashKey {
    fn name(&self) -> &'static str {
        "Lint/DuplicateHashKey"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Extract all `word:` patterns from the line (new hash syntax)
            // Also detect `"word" =>` or `:word =>` patterns
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut seen: HashSet<String> = HashSet::new();
            let mut j = 0;

            while j < len {
                let b = bytes[j];
                // Look for `word:` patterns (new hash key syntax)
                if b.is_ascii_alphabetic() || b == b'_' {
                    let key_start = j;
                    while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                        j += 1;
                    }
                    let key_end = j;

                    if j < len && bytes[j] == b':' && (j + 1 >= len || bytes[j + 1] != b':') {
                        // This is a hash key `word:`
                        let key = line[key_start..key_end].to_string();
                        if seen.contains(&key) {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            let abs_pos = (line_start + key_start) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!("Duplicate hash key `{}`.", key),
                                range: TextRange::new(abs_pos, abs_pos + key.len() as u32),
                                severity: Severity::Warning,
                            });
                        } else {
                            seen.insert(key);
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

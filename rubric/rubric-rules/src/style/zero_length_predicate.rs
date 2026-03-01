use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ZeroLengthPredicate;

const PATTERNS: &[&str] = &[".length == 0", ".size == 0", ".count == 0",
                             ".length.zero?", ".size.zero?"];

impl Rule for ZeroLengthPredicate {
    fn name(&self) -> &'static str {
        "Style/ZeroLengthPredicate"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            for pattern in PATTERNS {
                if let Some(pos) = line.find(pattern) {
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let abs_pos = (line_start + pos) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Use `.empty?` instead of `{}`.", pattern),
                        range: TextRange::new(abs_pos, abs_pos + pattern.len() as u32),
                        severity: Severity::Warning,
                    });
                    break; // one violation per line per check
                }
            }
        }

        diags
    }
}

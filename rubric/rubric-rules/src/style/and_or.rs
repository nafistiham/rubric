use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AndOr;

impl Rule for AndOr {
    fn name(&self) -> &'static str {
        "Style/AndOr"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect ` and ` or ` or ` in mid-expression context
            // Simple: find the patterns with spaces around them
            for (pattern, kw_len) in &[(" and ", 3usize), (" or ", 2usize)] {
                let mut search_start = 0usize;
                while let Some(pos) = line[search_start..].find(pattern) {
                    let abs_pos = search_start + pos;
                    // The keyword starts after the leading space
                    let kw_start = abs_pos + 1;
                    let line_start = ctx.line_start_offsets[i] as usize;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Use `&&`/`||` instead of `and`/`or` keyword."
                        ),
                        range: TextRange::new(
                            (line_start + kw_start) as u32,
                            (line_start + kw_start + kw_len) as u32,
                        ),
                        severity: Severity::Warning,
                    });
                    search_start = abs_pos + pattern.len();
                    if search_start >= line.len() { break; }
                }
            }
        }

        diags
    }
}

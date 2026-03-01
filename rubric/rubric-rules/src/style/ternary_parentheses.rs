use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TernaryParentheses;

impl Rule for TernaryParentheses {
    fn name(&self) -> &'static str {
        "Style/TernaryParentheses"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `(...) ?` pattern — opening paren, content, closing paren, space, `?`
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;

            while j < len {
                if bytes[j] == b'(' {
                    // Find matching `)`
                    let open_pos = j;
                    let mut depth = 1i32;
                    let mut k = j + 1;
                    while k < len && depth > 0 {
                        if bytes[k] == b'(' { depth += 1; }
                        else if bytes[k] == b')' { depth -= 1; }
                        k += 1;
                    }
                    // k is now after the closing `)`
                    let close_pos = k - 1;
                    // Check if `) ?` follows
                    if depth == 0 && close_pos + 1 < len && bytes[close_pos + 1] == b' '
                        && close_pos + 2 < len && bytes[close_pos + 2] == b'?' {
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Unnecessary parentheses around ternary condition.".into(),
                            range: TextRange::new(line_start + open_pos as u32, line_start + (close_pos + 1) as u32),
                            severity: Severity::Warning,
                        });
                    }
                    j = k;
                    continue;
                }
                j += 1;
            }
        }

        diags
    }
}

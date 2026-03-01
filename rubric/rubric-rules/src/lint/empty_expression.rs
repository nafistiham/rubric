use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyExpression;

impl Rule for EmptyExpression {
    fn name(&self) -> &'static str {
        "Lint/EmptyExpression"
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
            let mut j = 0;

            while j < len {
                if bytes[j] == b'(' {
                    let open_pos = j;
                    j += 1;
                    // Skip whitespace
                    while j < len && (bytes[j] == b' ' || bytes[j] == b'\t') {
                        j += 1;
                    }
                    if j < len && bytes[j] == b')' {
                        // Empty parentheses — but skip if preceded by a word char (method call)
                        let preceded_by_word = open_pos > 0 && (bytes[open_pos - 1].is_ascii_alphanumeric() || bytes[open_pos - 1] == b'_');
                        if !preceded_by_word {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Empty expression `()` has no value.".into(),
                                range: TextRange::new((line_start + open_pos) as u32, (line_start + j + 1) as u32),
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

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AmbiguousOperator;

impl Rule for AmbiguousOperator {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousOperator"
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
                let b = bytes[j];
                // Look for ` *word` or ` &word` pattern (space before * or & then word char)
                if b == b' ' && j + 2 < len {
                    let next = bytes[j + 1];
                    if (next == b'*' || next == b'&') && j + 2 < len {
                        let after_op = bytes[j + 2];
                        if after_op.is_ascii_alphabetic() || after_op == b'_' {
                            // Check that we're in a method call context (prev char is word/paren)
                            let prev_ok = j > 0 && (bytes[j - 1].is_ascii_alphanumeric() || bytes[j - 1] == b'_' || bytes[j - 1] == b')');
                            if prev_ok {
                                let line_start = ctx.line_start_offsets[i] as usize;
                                let op_pos = (line_start + j + 1) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!(
                                        "Ambiguous `{}` operator. Use parentheses to clarify intent.",
                                        next as char
                                    ),
                                    range: TextRange::new(op_pos, op_pos + 1),
                                    severity: Severity::Warning,
                                });
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

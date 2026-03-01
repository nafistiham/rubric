use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct ParenthesesAsGroupedExpression;

impl Rule for ParenthesesAsGroupedExpression {
    fn name(&self) -> &'static str {
        "Lint/ParenthesesAsGroupedExpression"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = trimmed.as_bytes();
            let n = bytes.len();
            let mut pos = 0;

            // Skip leading identifier (method name)
            while pos < n && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }

            // If we have an identifier followed by ` (` (space then open paren)
            if pos > 0 && pos < n && bytes[pos] == b' ' {
                let mut j = pos + 1;
                while j < n && bytes[j] == b' ' { j += 1; }
                if j < n && bytes[j] == b'(' {
                    // Check preceding chars are not keywords that would be ok
                    let method_name = std::str::from_utf8(&bytes[..pos]).unwrap_or("");
                    // Skip common Ruby keywords
                    if !matches!(method_name, "if" | "unless" | "while" | "until" | "return"
                        | "and" | "or" | "not" | "do" | "end" | "def" | "class" | "module") {
                        let indent = line.len() - trimmed.len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let space_pos = (line_start + indent + pos) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Avoid space between method name and `(`; it looks like a grouped expression.".into(),
                            range: TextRange::new(space_pos, space_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: String::new(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

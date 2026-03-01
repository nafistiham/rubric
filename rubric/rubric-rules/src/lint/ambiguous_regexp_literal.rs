use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AmbiguousRegexpLiteral;

impl Rule for AmbiguousRegexpLiteral {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousRegexpLiteral"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect method call followed by space then `/` (regexp without parens)
            // Patterns like `p /` or `puts /` or `print /`
            let bytes = trimmed.as_bytes();
            let n = bytes.len();
            let mut pos = 0;

            // Skip leading identifier (method name)
            while pos < n && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }

            // Check if followed by space then `/`
            if pos > 0 && pos < n && bytes[pos] == b' ' {
                let mut j = pos + 1;
                while j < n && bytes[j] == b' ' { j += 1; }
                if j < n && bytes[j] == b'/' {
                    // Make sure it's not `//` (empty regexp is still a regexp but less ambiguous)
                    let indent = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let flag_pos = (line_start + indent + j) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Ambiguous regexp literal; wrap in parentheses to clarify.".into(),
                        range: TextRange::new(flag_pos, flag_pos + 1),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

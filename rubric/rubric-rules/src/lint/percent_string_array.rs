use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct PercentStringArray;

impl Rule for PercentStringArray {
    fn name(&self) -> &'static str {
        "Lint/PercentStringArray"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = ctx.source;
        let bytes = source.as_bytes();
        let n = bytes.len();
        let mut i = 0;

        while i < n {
            // Look for %w[ or %W[
            if bytes[i] == b'%'
                && i + 1 < n
                && (bytes[i + 1] == b'w' || bytes[i + 1] == b'W')
                && i + 2 < n
                && bytes[i + 2] == b'['
            {
                let literal_start = i;
                i += 3; // skip %w[

                // Collect content until matching ]
                let content_start = i;
                let mut depth = 1usize;
                while i < n && depth > 0 {
                    match bytes[i] {
                        b'[' => depth += 1,
                        b']' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                // content is bytes[content_start..i-1]
                let content_end = if depth == 0 { i - 1 } else { i };
                let content =
                    std::str::from_utf8(&bytes[content_start..content_end]).unwrap_or("");

                // Split by whitespace and check each token for commas
                for token in content.split_ascii_whitespace() {
                    if token.contains(',') {
                        // Find the position of this token in source for accurate range
                        let token_offset = content
                            .find(token)
                            .map(|p| content_start + p)
                            .unwrap_or(literal_start);
                        let start = token_offset as u32;
                        let end = (token_offset + token.len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message:
                                "Within a `%w`/`%W` literal, `,` is just a character, not a separator."
                                    .into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                        break; // one diag per literal
                    }
                }

                continue;
            }
            i += 1;
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ForLoop;

impl Rule for ForLoop {
    fn name(&self) -> &'static str {
        "Style/For"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc = false;

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Skip heredoc bodies
            if in_heredoc {
                if !trimmed.is_empty()
                    && trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    in_heredoc = false;
                }
                continue;
            }

            // Detect heredoc openings
            if contains_heredoc_opener(line) {
                in_heredoc = true;
                // Fall through: still check the current line for `for`
            }

            // Check for `for <vars> in <expr>` pattern.
            // The line (after stripping indentation) must start with `for ` (word boundary)
            // and must contain ` in ` somewhere after.
            if trimmed.starts_with("for ") {
                // Ensure the content after `for ` contains ` in ` (i.e. it is a for-in loop)
                let rest = &trimmed[4..]; // after "for "
                if rest.contains(" in ") || rest.starts_with("in ") {
                    let line_start = ctx.line_start_offsets[i] as u32;
                    let line_end = line_start + line.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Prefer `each` over `for`.".into(),
                        range: TextRange::new(line_start, line_end),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

/// Returns `true` if the line contains a heredoc opening (`<<`, `<<-`, `<<~`).
fn contains_heredoc_opener(line: &str) -> bool {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut j = 0;
    while j < n {
        if bytes[j] == b'<' && j + 1 < n && bytes[j + 1] == b'<' {
            let rest_start = if j + 2 < n && (bytes[j + 2] == b'-' || bytes[j + 2] == b'~') {
                j + 3
            } else {
                j + 2
            };
            if rest_start < n
                && (bytes[rest_start].is_ascii_alphabetic() || bytes[rest_start] == b'_' || bytes[rest_start] == b'\'' || bytes[rest_start] == b'"')
            {
                return true;
            }
        }
        j += 1;
    }
    false
}

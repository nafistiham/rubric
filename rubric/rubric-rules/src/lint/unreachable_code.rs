use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UnreachableCode;

const TERMINATORS: &[&str] = &["return", "raise", "break", "next"];

impl Rule for UnreachableCode {
    fn name(&self) -> &'static str {
        "Lint/UnreachableCode"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            // Skip comments and blanks
            if trimmed.starts_with('#') || trimmed.is_empty() {
                i += 1;
                continue;
            }

            // Check if this line starts with a terminator keyword
            let is_terminator = TERMINATORS.iter().any(|kw| {
                trimmed.starts_with(kw) && (
                    trimmed.len() == kw.len()
                    || !trimmed.as_bytes().get(kw.len()).map(|b| b.is_ascii_alphanumeric() || *b == b'_').unwrap_or(false)
                )
            });

            // Guard clauses: "return if condition" / "return unless condition" — code after IS reachable
            if is_terminator && (trimmed.contains(" if ") || trimmed.contains(" unless ")) {
                i += 1;
                continue;
            }

            if is_terminator && i + 1 < n {
                // Look at next non-blank line at same indentation
                let mut j = i + 1;
                while j < n && lines[j].trim().is_empty() {
                    j += 1;
                }
                if j < n {
                    let next_line = &lines[j];
                    let next_trimmed = next_line.trim_start();
                    let next_indent = next_line.len() - next_trimmed.len();

                    // If next line is at same indentation and is not `end`/`else`/`elsif`/`rescue`/`ensure`
                    let is_block_end = next_trimmed == "end" || next_trimmed.starts_with("end ")
                        || next_trimmed == "else" || next_trimmed.starts_with("elsif ")
                        || next_trimmed == "rescue" || next_trimmed.starts_with("rescue ")
                        || next_trimmed == "ensure" || next_trimmed.starts_with("when ");

                    if next_indent == indent && !is_block_end && !next_trimmed.is_empty() {
                        let line_start = ctx.line_start_offsets[j];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Unreachable code after `return`/`raise`/`break`/`next`.".into(),
                            range: TextRange::new(line_start, line_start + next_trimmed.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            i += 1;
        }

        diags
    }
}

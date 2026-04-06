use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLineAfterGuardClause;

const GUARD_PREFIXES: &[&str] = &["return ", "raise ", "next ", "break ", "throw "];

/// Returns true if the line is a guard clause (starts with a guard keyword)
fn is_guard_clause(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return false;
    }
    if !GUARD_PREFIXES.iter().any(|p| trimmed.starts_with(p)) {
        return false;
    }
    // A guard clause that is a multi-line statement continuation should not be
    // flagged — the statement hasn't finished on this line.
    // Continuation indicators: trailing `,`, `\`, unclosed `[`, `(`, heredoc `<<`.
    if trimmed.ends_with(',')
        || trimmed.ends_with('\\')
        || trimmed.ends_with('[')
        || trimmed.ends_with('(')
        || trimmed.contains("<<~")
        || trimmed.contains("<<-")
        || (trimmed.contains("<<") && !trimmed.contains("<<>>"))
    {
        return false;
    }
    true
}

/// Returns true if the line is one that would terminate a guard clause section
fn is_terminator(line: &str) -> bool {
    let t = line.trim();
    t == "end"
        || t.starts_with("end ")
        || t == "else"
        || t.starts_with("else ")
        || t == "elsif"
        || t.starts_with("elsif ")
        || t == "rescue"
        || t.starts_with("rescue ")
        || t == "ensure"
        || t == "when"
        || t.starts_with("when ")
}

impl Rule for EmptyLineAfterGuardClause {
    fn name(&self) -> &'static str {
        "Layout/EmptyLineAfterGuardClause"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 0..n {
            if !is_guard_clause(&lines[i]) {
                continue;
            }

            // Guard clauses may span multiple lines when the condition is joined
            // by `&&` / `||` continuations, e.g.:
            //   return unless foo? &&
            //                  bar? &&
            //                  baz?
            // Advance `last` to the final continuation line before checking for
            // the required blank line.
            let mut last = i;
            while last + 1 < n {
                let t = lines[last].trim();
                if t.ends_with("&&") || t.ends_with("||") {
                    last += 1;
                } else {
                    break;
                }
            }

            // Look at next line after the full (possibly multi-line) guard clause
            if last + 1 >= n {
                continue;
            }
            let next = &lines[last + 1];
            let next_trimmed = next.trim();

            // If next line is blank, we're fine
            if next_trimmed.is_empty() {
                continue;
            }

            // If next line is a terminator, we're fine
            if is_terminator(next) {
                continue;
            }

            // If next non-blank, non-comment line is also a guard clause or
            // terminator, no blank line is needed (consecutive guards are OK).
            let next_meaningful = (last + 1..n)
                .map(|k| lines[k].trim())
                .find(|t| !t.is_empty() && !t.starts_with('#'));
            if let Some(t) = next_meaningful {
                let fake_line = format!("  {}", t);
                if is_guard_clause(&fake_line) || is_terminator(&fake_line) {
                    continue;
                }
            }

            let line_start = ctx.line_start_offsets[i];
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Add empty line after guard clause.".into(),
                range: TextRange::new(line_start, line_start + lines[i].len() as u32),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

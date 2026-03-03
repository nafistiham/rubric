use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AndOr;

/// Flow-control keywords that, when they follow `and`/`or`, mark the expression
/// as idiomatic flow-control rather than a boolean operation.
/// RuboCop `conditionals` style allows these patterns (e.g. `x or raise Y`,
/// `do_thing and return`).
const FLOW_CONTROL_KEYWORDS: &[&str] = &["raise", "return", "next", "break", "fail"];

/// Returns true when the text immediately following `and`/`or ` (i.e. the rest
/// of the line starting right after the trailing space of the pattern) begins
/// with a flow-control keyword.
fn is_flow_control_rhs(rest: &str) -> bool {
    let word = rest.trim_start();
    FLOW_CONTROL_KEYWORDS.iter().any(|&kw| {
        word.starts_with(kw)
            && word[kw.len()..]
                .chars()
                .next()
                .map_or(true, |c| !c.is_alphanumeric() && c != '_')
    })
}

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

            // Detect ` and ` or ` or ` used as boolean operators.
            // Skip occurrences where the RHS is a flow-control keyword
            // (e.g. `x or raise Y`, `do_thing and return`) — those are
            // idiomatic Ruby flow control, not boolean operations.
            for (pattern, kw_len) in &[(" and ", 3usize), (" or ", 2usize)] {
                let mut search_start = 0usize;
                while let Some(pos) = line[search_start..].find(pattern) {
                    let abs_pos = search_start + pos;
                    // The text that follows the full pattern (including its trailing space)
                    let after_pattern = &line[abs_pos + pattern.len()..];

                    if is_flow_control_rhs(after_pattern) {
                        // Flow-control usage — skip, not a boolean operator violation.
                        search_start = abs_pos + pattern.len();
                        if search_start >= line.len() {
                            break;
                        }
                        continue;
                    }

                    // The keyword starts after the leading space
                    let kw_start = abs_pos + 1;
                    let line_start = ctx.line_start_offsets[i] as usize;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `&&`/`||` instead of `and`/`or` keyword.".to_string(),
                        range: TextRange::new(
                            (line_start + kw_start) as u32,
                            (line_start + kw_start + kw_len) as u32,
                        ),
                        severity: Severity::Warning,
                    });
                    search_start = abs_pos + pattern.len();
                    if search_start >= line.len() {
                        break;
                    }
                }
            }
        }

        diags
    }
}

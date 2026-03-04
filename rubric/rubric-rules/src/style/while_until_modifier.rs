use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct WhileUntilModifier;

/// Returns true if the condition string contains a plain assignment `=` — not `==`, `!=`,
/// `<=`, `>=`, `=>`, `=~`, `+=`, `-=`, `*=`, `/=`, etc.
fn condition_has_assignment(cond: &str) -> bool {
    let bytes = cond.as_bytes();
    let n = bytes.len();
    for i in 0..n {
        if bytes[i] == b'=' {
            let prev_ok = i == 0
                || !matches!(
                    bytes[i - 1],
                    b'!' | b'<' | b'>' | b'=' | b'~' | b'+' | b'-' | b'*' | b'/'
                );
            let next_ok = i + 1 >= n || bytes[i + 1] != b'=';
            if prev_ok && next_ok {
                return true;
            }
        }
    }
    false
}

impl Rule for WhileUntilModifier {
    fn name(&self) -> &'static str {
        "Style/WhileUntilModifier"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Detect 3-line pattern: `while/until <cond>` / `  <single_stmt>` / `end`
        let mut i = 0;
        while i + 2 < n {
            let line0 = lines[i].trim();
            let line1 = lines[i + 1].trim();
            let line2 = lines[i + 2].trim();

            let is_while = line0.starts_with("while ") || line0 == "while";
            let is_until = line0.starts_with("until ") || line0 == "until";

            if (is_while || is_until) && line2 == "end" {
                // Skip if the condition contains an assignment: `while x = expr` cannot
                // be converted to modifier form because the assignment IS the condition.
                let kw_prefix = if is_while { "while " } else { "until " };
                let condition = if line0.len() > kw_prefix.len() { &line0[kw_prefix.len()..] } else { "" };
                if condition_has_assignment(condition) {
                    i += 1;
                    continue;
                }

                let adds_depth = line1.starts_with("if ") || line1.starts_with("unless ")
                    || line1.starts_with("while ") || line1.starts_with("until ")
                    || line1.starts_with("for ") || line1.starts_with("def ")
                    || line1.starts_with("class ") || line1.starts_with("module ")
                    || line1.starts_with("begin") || line1.ends_with(" do")
                    || line1.is_empty();

                if !adds_depth {
                    let line_start = ctx.line_start_offsets[i];
                    let keyword_len = 5u32; // "while" and "until" are both 5 chars
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use modifier form instead of multi-line `while`/`until`.".into(),
                        range: TextRange::new(line_start, line_start + keyword_len),
                        severity: Severity::Warning,
                    });
                }
            }
            i += 1;
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ParenthesesAroundCondition;

/// Keywords that introduce a condition block.
const CONDITION_KEYWORDS: &[&str] = &["if ", "while ", "until ", "unless "];

impl Rule for ParenthesesAroundCondition {
    fn name(&self) -> &'static str {
        "Style/ParenthesesAroundCondition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines.
            if trimmed.starts_with('#') {
                continue;
            }

            // Only match lines where the keyword is the first token (statement
            // position). A keyword in the middle of a line (modifier form) is
            // not a condition-introducing keyword, so we skip those.
            let mut matched_keyword = false;
            for kw in CONDITION_KEYWORDS {
                if !trimmed.starts_with(kw) {
                    continue;
                }
                // The remainder after the keyword (with optional extra spaces).
                let after_kw = trimmed[kw.len()..].trim_start();

                // The condition must start with `(`.
                if !after_kw.starts_with('(') {
                    continue;
                }

                // The `(` must not be a method call. A method-call `(` is
                // immediately preceded by a word character (letter, digit, `_`,
                // `?`, `!`) with no space in between. Here we already stripped
                // any spaces with `trim_start()`, so if `after_kw` starts with
                // `(` right after a keyword, it is a redundant condition paren.
                //
                // Additionally guard against `if (x) method_call(y)` — the
                // `(` we are looking at is the first `(` after the keyword, not
                // a subsequent method-call paren. That is fine since we matched
                // on `after_kw.starts_with('(')`.

                // Find the byte offset of the `(` in the original line.
                let indent = line.len() - trimmed.len();
                let kw_len = kw.trim_end().len(); // length without trailing space
                let after_kw_offset = indent + kw.len() + (trimmed[kw.len()..].len() - after_kw.len());
                let paren_offset = after_kw_offset; // `(` is the first char of after_kw

                let line_start = ctx.line_start_offsets[i] as usize;
                let start = (line_start + paren_offset) as u32;
                let end = start + 1;

                let _ = kw_len; // suppress unused warning

                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Don't use parentheses around the condition of an if.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });

                matched_keyword = true;
                break;
            }

            let _ = matched_keyword;
        }

        diags
    }
}

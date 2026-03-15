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

                // Only flag when the parens wrap the ENTIRE condition.
                // Find the matching `)` and verify:
                // 1. Nothing significant follows it (sub-expression grouping → skip)
                // 2. The interior does NOT contain a bare assignment (AllowSafeAssignment)
                {
                    let inner = &after_kw[1..]; // bytes after the `(`
                    let bytes = inner.as_bytes();
                    let mut depth = 1i32;
                    let mut close_pos = None;
                    let mut has_assignment = false;
                    let mut j = 0;
                    while j < bytes.len() {
                        match bytes[j] {
                            b'(' => depth += 1,
                            b')' => {
                                depth -= 1;
                                if depth == 0 {
                                    close_pos = Some(j);
                                    break;
                                }
                            }
                            b'=' if depth == 1 => {
                                let prev = if j > 0 { bytes[j - 1] } else { 0 };
                                let next = bytes.get(j + 1).copied().unwrap_or(0);
                                if !matches!(prev, b'!' | b'<' | b'>' | b'=')
                                    && !matches!(next, b'=' | b'>')
                                {
                                    has_assignment = true;
                                }
                            }
                            _ => {}
                        }
                        j += 1;
                    }
                    // AllowSafeAssignment: parens required around assignments.
                    if has_assignment {
                        continue;
                    }
                    // If matching `)` found, check what follows it.
                    if let Some(cp) = close_pos {
                        let after_close = inner[cp + 1..].trim_start();
                        // Something non-trivial follows → parens are sub-expression.
                        if !after_close.is_empty() && !after_close.starts_with('#') {
                            continue;
                        }
                    }
                }

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

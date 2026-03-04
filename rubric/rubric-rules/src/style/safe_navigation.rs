use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SafeNavigation;

impl Rule for SafeNavigation {
    fn name(&self) -> &'static str {
        "Style/SafeNavigation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();

            // Look for `<var> && <var>.` pattern where var names match
            // Scan for `&&` first
            let mut j = 0;
            while j + 3 < len {
                if bytes[j] == b'&' && bytes[j+1] == b'&' {
                    // Found `&&` at position j
                    // Look backwards for the variable name before `&&`
                    // (skip space before `&&`)
                    let mut left_end = j;
                    while left_end > 0 && bytes[left_end-1] == b' ' {
                        left_end -= 1;
                    }
                    // Read word chars backwards
                    let mut left_start = left_end;
                    while left_start > 0 && (bytes[left_start-1].is_ascii_alphanumeric() || bytes[left_start-1] == b'_') {
                        left_start -= 1;
                    }

                    if left_start >= left_end {
                        j += 2;
                        continue;
                    }

                    let var_name = &bytes[left_start..left_end];

                    // Look forward past `&&` and space for the same var name followed by `.`
                    let mut right_start = j + 2;
                    while right_start < len && bytes[right_start] == b' ' {
                        right_start += 1;
                    }

                    // Check if what follows is `<var_name>.`
                    let var_len = var_name.len();
                    if right_start + var_len + 1 <= len
                        && &bytes[right_start..right_start + var_len] == var_name
                        && bytes[right_start + var_len] == b'.'
                    {
                        // Skip past var_name.method to check what follows.
                        let method_start = right_start + var_len + 1;
                        let mut method_end = method_start;
                        while method_end < len
                            && (bytes[method_end].is_ascii_alphanumeric()
                                || bytes[method_end] == b'_'
                                || bytes[method_end] == b'?'
                                || bytes[method_end] == b'!')
                        {
                            method_end += 1;
                        }
                        // Skip argument parens if any: var.method(...)
                        if method_end < len && bytes[method_end] == b'(' {
                            let mut depth = 1;
                            method_end += 1;
                            while method_end < len && depth > 0 {
                                if bytes[method_end] == b'(' {
                                    depth += 1;
                                } else if bytes[method_end] == b')' {
                                    depth -= 1;
                                }
                                method_end += 1;
                            }
                        }
                        // Check what follows the method call — if a comparison or logical
                        // operator trails the expression, converting to `&.` would change
                        // semantics (nil vs false difference in short-circuit evaluation).
                        let rest = line[method_end..].trim_start();
                        let has_trailing_op = rest.starts_with("!=")
                            || rest.starts_with("==")
                            || rest.starts_with("<=")
                            || rest.starts_with(">=")
                            || rest.starts_with(" < ")
                            || rest.starts_with(" > ")
                            || rest.starts_with("&&")
                            || rest.starts_with("||")
                            || rest.starts_with("&.") // chained safe nav
                            || (!rest.is_empty() && rest.as_bytes()[0] == b'.');
                        if has_trailing_op {
                            j += 2;
                            continue;
                        }

                        let pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "Use safe navigation operator (`&.`) instead of `{} && {}.`.",
                                std::str::from_utf8(var_name).unwrap_or("?"),
                                std::str::from_utf8(var_name).unwrap_or("?"),
                            ),
                            range: TextRange::new(pos, pos + 2),
                            severity: Severity::Warning,
                        });
                    }

                    j += 2;
                } else {
                    j += 1;
                }
            }
        }

        diags
    }
}

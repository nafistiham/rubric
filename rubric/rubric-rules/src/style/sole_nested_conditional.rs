use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SoleNestedConditional;

impl SoleNestedConditional {
    /// Returns true if the trimmed line opens an `if` or `unless` multi-line block.
    fn is_conditional_start(trimmed: &str) -> bool {
        trimmed.starts_with("if ") || trimmed.starts_with("unless ")
    }

    /// Returns the number of leading spaces in a raw line.
    fn indent_of(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }

    /// Analyse the block whose header is at `lines[start]` with `base_indent`
    /// leading spaces.
    ///
    /// Scans forward until the matching `end` at `base_indent`.
    ///
    /// Returns `None` if the block is not properly terminated.
    ///
    /// Returns `Some((end_idx, has_else_elsif, sole_body_line))` where:
    /// - `end_idx`        — index of the `end` line
    /// - `has_else_elsif` — true if any `else`/`elsif` was at `base_indent`
    /// - `sole_body_line` — `Some(idx)` when the body contains exactly one
    ///                      distinct statement at the immediate child indent
    fn analyse_block(
        lines: &[&str],
        start: usize,
        base_indent: usize,
    ) -> Option<(usize, bool, Option<usize>)> {
        let n = lines.len();
        let mut i = start + 1;
        let mut has_else_elsif = false;
        let mut first_body_idx: Option<usize> = None;
        let mut child_indent: Option<usize> = None;
        let mut child_line_count = 0usize;
        // Tracks nesting depth for keyword blocks inside the body.
        let mut depth = 0usize;

        while i < n {
            let trimmed = lines[i].trim();
            if trimmed.is_empty() {
                i += 1;
                continue;
            }

            let indent = Self::indent_of(lines[i]);

            if depth == 0 && indent == base_indent {
                if trimmed == "end" {
                    let sole = first_body_idx.filter(|_| child_line_count == 1);
                    return Some((i, has_else_elsif, sole));
                }
                if trimmed.starts_with("else") || trimmed.starts_with("elsif") {
                    has_else_elsif = true;
                }
            } else if indent > base_indent {
                let ci = *child_indent.get_or_insert(indent);

                if first_body_idx.is_none() {
                    first_body_idx = Some(i);
                }

                if depth == 0 && indent == ci {
                    child_line_count += 1;
                    // Descend into nested keyword blocks so their `end` is not
                    // confused with the outer block's `end`.
                    if Self::opens_block(trimmed) {
                        depth += 1;
                    }
                } else if depth > 0 {
                    if trimmed == "end" {
                        depth -= 1;
                    } else if Self::opens_block(trimmed) {
                        depth += 1;
                    }
                }
            }

            i += 1;
        }

        None // unterminated block
    }

    /// True when a trimmed line opens a keyword block that is closed by `end`.
    fn opens_block(trimmed: &str) -> bool {
        trimmed.starts_with("if ")
            || trimmed.starts_with("unless ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("module ")
            || trimmed.starts_with("begin")
            || trimmed.starts_with("case ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("until ")
            || trimmed.starts_with("for ")
            || trimmed == "begin"
    }
}

impl Rule for SoleNestedConditional {
    fn name(&self) -> &'static str {
        "Style/SoleNestedConditional"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0usize;

        while i < n {
            let line = lines[i];
            let trimmed = line.trim_start();

            if !Self::is_conditional_start(trimmed) {
                i += 1;
                continue;
            }

            // Skip modifier / `then` forms (one-liners like `if foo then bar end`).
            if trimmed.contains(" then ") || trimmed.ends_with(" then") {
                i += 1;
                continue;
            }

            let outer_indent = Self::indent_of(line);

            let Some((outer_end, outer_has_else, sole_nested)) =
                Self::analyse_block(lines, i, outer_indent)
            else {
                i += 1;
                continue;
            };

            if outer_has_else {
                i += 1;
                continue;
            }

            let Some(inner_start) = sole_nested else {
                i += 1;
                continue;
            };

            let inner_trimmed = lines[inner_start].trim_start();
            if !Self::is_conditional_start(inner_trimmed) {
                i += 1;
                continue;
            }

            if inner_trimmed.contains(" then ") || inner_trimmed.ends_with(" then") {
                i += 1;
                continue;
            }

            let inner_indent = Self::indent_of(lines[inner_start]);
            let Some((_inner_end, inner_has_else, _)) =
                Self::analyse_block(lines, inner_start, inner_indent)
            else {
                i += 1;
                continue;
            };

            if inner_has_else {
                i += 1;
                continue;
            }

            // Flag the outer conditional.
            let line_start = ctx.line_start_offsets[i] as usize;
            let pos = (line_start + outer_indent) as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Consider merging nested conditions into outer condition.".into(),
                range: TextRange::new(pos, pos + trimmed.len() as u32),
                severity: Severity::Warning,
            });

            // Advance past the outer block to avoid re-scanning inner lines.
            i = outer_end + 1;
        }

        diags
    }
}

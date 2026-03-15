use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CaseLikeIf;

impl CaseLikeIf {
    fn indent_of(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }

    /// Extract the left-hand side of an `<lhs> == <rhs>` comparison on a
    /// trimmed `if` or `elsif` line.  Returns `None` if the pattern does not
    /// match (compound conditions, different operators, etc.).
    fn extract_lhs(trimmed: &str) -> Option<&str> {
        // Strip leading keyword and whitespace.
        let rest = if trimmed.starts_with("if ") {
            &trimmed[3..]
        } else if trimmed.starts_with("elsif ") {
            &trimmed[6..]
        } else {
            return None;
        };

        // Find ` == ` in the expression.
        let eq_pos = rest.find(" == ")?;
        let lhs = rest[..eq_pos].trim();

        // Reject compound conditions that would make the LHS itself complex.
        if lhs.contains("&&")
            || lhs.contains("||")
            || lhs.contains(" and ")
            || lhs.contains(" or ")
        {
            return None;
        }

        // The RHS must not be a compound expression either.
        let rhs = rest[eq_pos + 4..].trim();
        if rhs.contains(" && ")
            || rhs.contains(" || ")
            || rhs.contains(" and ")
            || rhs.contains(" or ")
        {
            return None;
        }

        Some(lhs)
    }
}

impl Rule for CaseLikeIf {
    fn name(&self) -> &'static str {
        "Style/CaseLikeIf"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0usize;

        while i < n {
            let line = lines[i];
            let trimmed = line.trim_start();

            // Look for an `if <lhs> == <rhs>` line.
            if !trimmed.starts_with("if ") {
                i += 1;
                continue;
            }

            let Some(if_lhs) = Self::extract_lhs(trimmed) else {
                i += 1;
                continue;
            };

            // Scan forward collecting `elsif <lhs> == <rhs>` lines that share
            // the same LHS variable.
            let mut matching_branches = 1usize; // the `if` branch counts as 1
            let mut j = i + 1;

            while j < n {
                let jt = lines[j].trim_start();

                if jt.starts_with("elsif ") {
                    if let Some(elsif_lhs) = Self::extract_lhs(jt) {
                        if elsif_lhs == if_lhs {
                            matching_branches += 1;
                        } else {
                            // Different LHS — chain broken.
                            break;
                        }
                    } else {
                        // Complex `elsif` expression — chain broken.
                        break;
                    }
                } else if jt == "else" || jt == "end" {
                    break;
                }
                // Body lines between branches: advance past them.
                j += 1;
            }

            // Flag when there are at least 2 branches comparing the same LHS
            // (the `if` branch + at least one `elsif`).
            if matching_branches >= 2 {
                let indent = Self::indent_of(line);
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Convert if-elsif to case-when.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }

            i += 1;
        }

        diags
    }
}

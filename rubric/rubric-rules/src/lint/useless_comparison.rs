use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UselessComparison;

const OPS: &[&str] = &["==", "!=", "<=", ">=", "<", ">"];

impl Rule for UselessComparison {
    fn name(&self) -> &'static str {
        "Lint/UselessComparison"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // For each operator, look for `word op word` where both words are the same
            for op in OPS {
                let mut search_start = 0usize;
                while let Some(pos) = line[search_start..].find(op) {
                    let abs_pos = search_start + pos;

                    // Read left operand (word before op)
                    let before = &line[..abs_pos].trim_end();
                    let lhs_end = before.len();
                    let mut lhs_start = lhs_end;
                    let before_bytes = before.as_bytes();
                    while lhs_start > 0 && (before_bytes[lhs_start - 1].is_ascii_alphanumeric() || before_bytes[lhs_start - 1] == b'_') {
                        lhs_start -= 1;
                    }
                    let lhs = &before[lhs_start..lhs_end];

                    if lhs.is_empty() {
                        search_start = abs_pos + op.len();
                        continue;
                    }

                    // Read right operand (word after op)
                    let after_op = abs_pos + op.len();
                    let after = &line[after_op..].trim_start();
                    let mut rhs_end = 0;
                    let after_bytes = after.as_bytes();
                    while rhs_end < after.len() && (after_bytes[rhs_end].is_ascii_alphanumeric() || after_bytes[rhs_end] == b'_') {
                        rhs_end += 1;
                    }
                    let rhs = &after[..rhs_end];

                    // Skip if LHS has a receiver (char before lhs_start is `.`, `:`, or `@`)
                    // e.g. `thread.account_id`   — `.` method call receiver
                    // e.g. `Admin::AccountStatusesFilter` — `::` namespace separator
                    // e.g. `@account_id`          — `@` instance variable prefix
                    let lhs_has_receiver = lhs_start > 0
                        && matches!(
                            before_bytes.get(lhs_start.wrapping_sub(1)).copied(),
                            Some(b'.') | Some(b':') | Some(b'@')
                        );

                    // Skip if a binary operator precedes the LHS (with optional space).
                    // e.g. `computed_permissions & permissions != permissions` — the `&`
                    // makes the full LHS a compound expression, not just `permissions`.
                    let mut k = lhs_start;
                    while k > 0 && before_bytes[k - 1] == b' ' { k -= 1; }
                    let lhs_has_operator_prefix = k > 0
                        && matches!(
                            before_bytes[k - 1],
                            b'&' | b'|' | b'^' | b'+' | b'-' | b'*' | b'/' | b'%'
                        );

                    if !rhs.is_empty() && lhs == rhs && !lhs_has_receiver && !lhs_has_operator_prefix {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let op_pos = (line_start + abs_pos) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!("Comparing `{}` to itself is always true/false.", lhs),
                            range: TextRange::new(op_pos, op_pos + op.len() as u32),
                            severity: Severity::Warning,
                        });
                        break;
                    }

                    search_start = abs_pos + op.len();
                    if search_start >= line.len() { break; }
                }
            }
        }

        diags
    }
}

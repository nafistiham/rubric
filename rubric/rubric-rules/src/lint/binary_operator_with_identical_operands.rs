use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct BinaryOperatorWithIdenticalOperands;

/// Returns `true` if `pos` in `bytes` is inside a string literal (`"` or `'`).
/// Stops scanning at an unquoted `#` (comment).
fn in_string_at(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos && i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return false,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Operators to check, ordered longest-first so `&&` is matched before `&`,
/// `||` before `|`, `==` before `=`, etc.
const OPS: &[&str] = &[
    "&&", "||", "==", "!=", "<=", ">=", "<<", ">>",
    "<", ">", "+", "-", "*", "/", "&", "|", "^",
];

impl Rule for BinaryOperatorWithIdenticalOperands {
    fn name(&self) -> &'static str {
        "Lint/BinaryOperatorWithIdenticalOperands"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            'ops: for op in OPS {
                let op_bytes = op.as_bytes();
                let mut search = 0usize;

                while search + op_bytes.len() <= bytes.len() {
                    // Find next occurrence of the operator bytes
                    let found = bytes[search..]
                        .windows(op_bytes.len())
                        .position(|w| w == op_bytes);

                    let rel = match found {
                        Some(r) => r,
                        None => continue 'ops,
                    };
                    let abs_op = search + rel;

                    // Ensure the operator is not part of a longer operator.
                    // For single-char ops check the char before and after are not
                    // part of a two-char op already handled above.
                    if op_bytes.len() == 1 {
                        // Skip if the byte before forms a two-char op with this byte
                        let prev = abs_op.checked_sub(1).and_then(|p| bytes.get(p)).copied();
                        let next = bytes.get(abs_op + 1).copied();
                        let op_byte = op_bytes[0];

                        // Skip `<=`, `>=`, `!=`, `==`, `<<`, `>>`, `&&`, `||`
                        // that were already checked as two-char ops.
                        let is_extension = match op_byte {
                            b'<' => prev == Some(b'<') || next == Some(b'<') || next == Some(b'='),
                            b'>' => prev == Some(b'>') || next == Some(b'>') || next == Some(b'='),
                            b'&' => prev == Some(b'&') || next == Some(b'&'),
                            b'|' => prev == Some(b'|') || next == Some(b'|'),
                            b'+' | b'-' | b'*' | b'/' => {
                                // Skip `+=`, `-=`, `*=`, `/=`, `**`
                                next == Some(b'=') || (op_byte == b'*' && next == Some(b'*'))
                                    || (op_byte == b'*' && prev == Some(b'*'))
                            }
                            _ => false,
                        };

                        if is_extension {
                            search = abs_op + 1;
                            continue;
                        }
                    }

                    // Skip if this position is inside a string
                    if in_string_at(bytes, abs_op) {
                        search = abs_op + op_bytes.len();
                        continue;
                    }

                    // Read LHS: word immediately before the operator (trimming whitespace)
                    let before = &line[..abs_op];
                    let before_trimmed = before.trim_end();
                    let lhs_end = before_trimmed.len();
                    let before_bytes = before_trimmed.as_bytes();

                    let mut lhs_start = lhs_end;
                    while lhs_start > 0
                        && (before_bytes[lhs_start - 1].is_ascii_alphanumeric()
                            || before_bytes[lhs_start - 1] == b'_')
                    {
                        lhs_start -= 1;
                    }
                    let lhs = &before_trimmed[lhs_start..lhs_end];

                    if lhs.is_empty() {
                        search = abs_op + op_bytes.len();
                        continue;
                    }

                    // Skip if LHS has a receiver (`.`, `:`, `@`, `$` before it)
                    let lhs_has_receiver = lhs_start > 0
                        && matches!(
                            before_bytes.get(lhs_start - 1).copied(),
                            Some(b'.') | Some(b':') | Some(b'@') | Some(b'$')
                        );
                    if lhs_has_receiver {
                        search = abs_op + op_bytes.len();
                        continue;
                    }

                    // Read RHS: word immediately after the operator (trimming whitespace)
                    let after_op = abs_op + op_bytes.len();
                    let after = &line[after_op..];
                    let after_trimmed = after.trim_start();
                    let mut rhs_end = 0;
                    let after_bytes = after_trimmed.as_bytes();
                    while rhs_end < after_trimmed.len()
                        && (after_bytes[rhs_end].is_ascii_alphanumeric()
                            || after_bytes[rhs_end] == b'_')
                    {
                        rhs_end += 1;
                    }
                    let rhs = &after_trimmed[..rhs_end];

                    if rhs.is_empty() || lhs != rhs {
                        search = abs_op + op_bytes.len();
                        continue;
                    }

                    // Check that RHS is not followed by a `.` (receiver) — e.g. `foo.bar`
                    let rhs_abs_end = after_op
                        + (after.len() - after_trimmed.len()) // whitespace offset
                        + rhs_end;
                    let rhs_has_receiver = bytes.get(rhs_abs_end).copied() == Some(b'.');
                    if rhs_has_receiver {
                        search = abs_op + op_bytes.len();
                        continue;
                    }

                    let op_start = (line_start + abs_op) as u32;
                    let op_end = op_start + op_bytes.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Binary operator `{}` has identical operands.", op),
                        range: TextRange::new(op_start, op_end),
                        severity: Severity::Warning,
                    });

                    // Move past this operator to avoid double-reporting
                    search = abs_op + op_bytes.len();
                }
            }
        }

        diags
    }
}

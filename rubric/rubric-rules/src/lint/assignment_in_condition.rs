use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AssignmentInCondition;

impl Rule for AssignmentInCondition {
    fn name(&self) -> &'static str {
        "Lint/AssignmentInCondition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Only check lines that start with `if ` or `while `
            let has_cond_kw = trimmed.starts_with("if ") || trimmed.starts_with("while ")
                || trimmed.starts_with("unless ") || trimmed.starts_with("until ");

            if !has_cond_kw {
                continue;
            }

            // Scan for single `=` that's not `==`, `!=`, `<=`, `>=`, `=>`
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut paren_depth: i32 = 0;
            let mut j = 0;

            while j < len {
                let b = bytes[j];
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break,
                    None => {}
                }

                if b == b'(' { paren_depth += 1; j += 1; continue; }
                if b == b')' { paren_depth -= 1; j += 1; continue; }

                if b == b'=' {
                    // Check it's not `==`, `!=`, `<=`, `>=`, `=>`
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    let next = if j + 1 < len { bytes[j + 1] } else { 0 };

                    let is_comparison = next == b'=' || next == b'>' || next == b'~'  // ==, =>, =~
                        || prev == b'!' || prev == b'<' || prev == b'>' || prev == b'='  // !=, <=, >=, ==
                        || prev == b'|' || prev == b'&' || prev == b'+' || prev == b'-'  // ||=, &&=, +=, -=
                        || prev == b'*' || prev == b'/' || prev == b'%' || prev == b'^'  // *=, /=, %=, ^=
                        || prev == b'~'  // ~= (uncommon but safe to skip)
                        ;

                    if !is_comparison {
                        // `AllowSafeAssignment: true` (rubocop default): skip if inside parens,
                        // e.g. `if (x = foo)` — the parens signal intentionality.
                        if paren_depth > 0 {
                            j += 1;
                            continue;
                        }
                        // It's a single `=` outside parens — likely unintentional
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Assignment in condition. Did you mean `==`?".into(),
                            range: TextRange::new(line_start + j as u32, line_start + j as u32 + 1),
                            severity: Severity::Warning,
                        });
                        break;
                    }
                }
                j += 1;
            }
        }

        diags
    }
}

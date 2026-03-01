use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ConditionalAssignment;

impl Rule for ConditionalAssignment {
    fn name(&self) -> &'static str {
        "Style/ConditionalAssignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let trimmed = lines[i].trim();
            // Look for `if` or `unless` at the start of a line
            if !trimmed.starts_with("if ") && !trimmed.starts_with("unless ") {
                i += 1;
                continue;
            }

            // Find the structure: if ... \n <var> = <val> \n else \n <same_var> = <val2> \n end
            // We need at least 5 more lines: if, assign, else, assign, end
            if i + 4 >= n {
                i += 1;
                continue;
            }

            let if_line_idx = i;
            let then_line = lines[i + 1].trim();
            let else_line = lines[i + 2].trim();
            let else_assign = lines[i + 3].trim();
            let end_line = lines[i + 4].trim();

            if else_line != "else" || end_line != "end" {
                i += 1;
                continue;
            }

            // Extract variable from then branch assignment
            let then_var = extract_lhs_var(then_line);
            let else_var = extract_lhs_var(else_assign);

            if let (Some(tv), Some(ev)) = (then_var, else_var) {
                if tv == ev {
                    let indent = lines[if_line_idx].len() - lines[if_line_idx].trim_start().len();
                    let line_start = ctx.line_start_offsets[if_line_idx] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Use `{}` outside the conditional instead of assigning in both branches.",
                            tv
                        ),
                        range: TextRange::new(pos, pos + lines[if_line_idx].trim_start().len() as u32),
                        severity: Severity::Warning,
                    });
                    i += 5;
                    continue;
                }
            }

            i += 1;
        }

        diags
    }
}

fn extract_lhs_var(line: &str) -> Option<&str> {
    // Match `<identifier> = ` at start of trimmed line (not `==`)
    let eq_pos = line.find(" = ")?;
    let lhs = &line[..eq_pos];
    // lhs should be a simple identifier (no spaces)
    if lhs.contains(' ') || lhs.is_empty() {
        return None;
    }
    // Must start with lowercase or underscore
    let first = lhs.chars().next()?;
    if first.is_ascii_lowercase() || first == '_' {
        Some(lhs)
    } else {
        None
    }
}

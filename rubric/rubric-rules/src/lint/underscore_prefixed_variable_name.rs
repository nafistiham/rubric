use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UnderscorePrefixedVariableName;

impl Rule for UnderscorePrefixedVariableName {
    fn name(&self) -> &'static str {
        "Lint/UnderscorePrefixedVariableName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;
        let lines = &ctx.lines;
        let n = lines.len();

        // Collect `_var` assignments
        let mut underscore_vars: Vec<(String, usize)> = Vec::new(); // (name, line_idx)

        for i in 0..n {
            let trimmed = lines[i].trim_start();
            let t = trimmed.trim();
            if t.starts_with('#') { continue; }

            if let Some(eq_pos) = t.find(" = ") {
                let lhs = t[..eq_pos].trim();
                if lhs.starts_with('_') && lhs.len() > 1
                    && lhs.chars().nth(1).map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
                    underscore_vars.push((lhs.to_string(), i));
                }
            }
        }

        // Check if each `_var` is used elsewhere in the source
        for (var, assign_line) in &underscore_vars {
            let vb = var.as_bytes();
            let bb = src.as_bytes();
            let src_len = bb.len();
            let vn = vb.len();
            let mut used = false;

            let mut pos = 0;
            while pos + vn <= src_len {
                if &bb[pos..pos + vn] == vb {
                    let before_ok = pos == 0 || !bb[pos - 1].is_ascii_alphanumeric() && bb[pos - 1] != b'_';
                    let after_ok = pos + vn >= src_len || !bb[pos + vn].is_ascii_alphanumeric() && bb[pos + vn] != b'_';
                    if before_ok && after_ok {
                        // Check if this is NOT the assignment itself
                        // Find which line this position is on
                        let line_of_occurrence = ctx.line_start_offsets.partition_point(|&o| o as usize <= pos).saturating_sub(1);
                        if line_of_occurrence != *assign_line {
                            used = true;
                            break;
                        }
                    }
                }
                pos += 1;
            }

            if used {
                let indent = lines[*assign_line].len() - lines[*assign_line].trim_start().len();
                let line_start = ctx.line_start_offsets[*assign_line] as usize;
                let pos_out = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "Variable `{}` is prefixed with `_` but is actually used.",
                        var
                    ),
                    range: TextRange::new(pos_out, pos_out + var.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

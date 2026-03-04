use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

/// Returns true when `trimmed` is an endless method definition of the form:
///   `def name = expr`   or   `def name(args) = expr`
/// In both cases ` = ` (or ` =` at end-of-line) appears OUTSIDE of parentheses.
fn is_endless_def(trimmed: &str) -> bool {
    // Strip the leading `def ` prefix.
    let rest = match trimmed.strip_prefix("def ") {
        Some(r) => r,
        None => return false,
    };
    // Walk through `rest` tracking paren depth; if we encounter ` = ` at depth 0
    // the definition is an endless method.
    let mut depth = 0usize;
    let bytes = rest.as_bytes();
    let len = bytes.len();
    let mut j = 0;
    while j < len {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'=' if depth == 0 => {
                // Make sure this is ` = ` (assignment-style), not `==` or `!=`
                let prev_is_space = j > 0 && bytes[j - 1] == b' ';
                let next_is_space_or_end = j + 1 >= len || bytes[j + 1] != b'=';
                if prev_is_space && next_is_space_or_end {
                    return true;
                }
            }
            _ => {}
        }
        j += 1;
    }
    false
}

pub struct EmptyMethod;

impl Rule for EmptyMethod {
    fn name(&self) -> &'static str {
        "Style/EmptyMethod"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let trimmed = lines[i].trim_start();
            if trimmed.starts_with("def ") || trimmed == "def" {
                // Skip single-line forms that already have a body on the def line:
                //   `def foo; end`  — contains `; end` or ` end`
                //   `def foo; body` — has a semicolon (inline body separator)
                // Skip endless method forms (no body block needed):
                //   `def foo = expr` or `def foo(args) = expr`
                //   Detected by: ` = ` present and not inside parentheses
                //   (i.e., the last `)` before ` = ` is balanced or absent)
                let has_inline_body = trimmed.contains(" end")
                    || trimmed.ends_with(";end")
                    || trimmed.contains(';');
                let is_endless_method = is_endless_def(trimmed);
                if has_inline_body || is_endless_method {
                    i += 1;
                    continue;
                }
                // Check if the very next non-empty line is `end`
                if i + 1 < n {
                    let next = lines[i + 1].trim();
                    if next == "end" {
                        let indent = lines[i].len() - trimmed.len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        // Range covers from def to end of `end` line
                        let end_line_start = ctx.line_start_offsets[i + 1] as usize;
                        let end_line = &lines[i + 1];
                        let end_pos = (end_line_start + end_line.len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Empty method body; use `def foo; end` on a single line.".into(),
                            range: TextRange::new(pos, end_pos),
                            severity: Severity::Warning,
                        });
                        i += 2;
                        continue;
                    }
                }
            }
            i += 1;
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // We cannot easily reconstruct the def signature here from just the range,
        // so we return None (fix requires source context beyond what Diagnostic carries).
        // The fix is handled by the check_source logic above for reporting only.
        let _ = diag;
        None
    }
}

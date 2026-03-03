use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DefEndAlignment;

/// Returns true if `trimmed` is an endless method (`def foo = expr` / `def foo(x) = expr`).
fn is_endless_method(trimmed: &str) -> bool {
    let def_pos = match trimmed.find("def ") {
        Some(p) if p <= 20 => p,
        _ => return false,
    };
    let after_def = &trimmed[def_pos + 4..];
    let bytes = after_def.as_bytes();
    let n = bytes.len();
    let mut depth = 0i32;
    let mut i = 0;
    while i < n {
        match bytes[i] {
            b'(' => { depth += 1; }
            b')' => { depth -= 1; }
            b' ' if depth == 0 && i + 2 < n
                && bytes[i + 1] == b'='
                && bytes[i + 2] != b'='
                && bytes[i + 2] != b'>' => {
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Returns true if `trimmed` is a one-liner like `class Foo; end`.
fn is_one_liner(trimmed: &str) -> bool {
    trimmed.trim_end_matches(|c: char| c == ' ' || c == '\t').ends_with("; end")
}

impl Rule for DefEndAlignment {
    fn name(&self) -> &'static str {
        "Layout/DefEndAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Unified stack: (line_idx, indent, is_def)
        // is_def=true = def/class/module (alignment checked), is_def=false = inner construct
        let mut stack: Vec<(usize, usize, bool)> = Vec::new();

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            let t = trimmed.trim();

            // Exclude one-liners and endless methods from all stack tracking.
            let is_def_opener = !is_one_liner(t) && !is_endless_method(t) && (
                t.starts_with("def ") || t == "def"
                || t.starts_with("private def ") || t.starts_with("protected def ")
                || t.starts_with("class ") || t.starts_with("module ")
            );

            let is_inner_construct = !is_def_opener && !is_one_liner(t) && (
                t.starts_with("if ")
                || t.starts_with("unless ")
                || t.starts_with("while ")
                || t.starts_with("until ")
                || t == "begin"
                || t.starts_with("begin ")
                || t.starts_with("case ")
                || t.ends_with(" do")
                || t.contains(" do |")
                || t.contains(" do|")
                || t == "do"
            );

            // Inline conditional/begin assignment: `x = if cond` / `x ||= if` / `x = begin` / etc.
            // The `end` that closes these should NOT be compared to the enclosing def.
            let has_inline_conditional = !is_def_opener && !is_inner_construct && (
                // Any assignment variant (=, ||=, &&=, +=, etc.) followed by if/unless/case
                t.contains("= if ") || t.ends_with("= if")
                || t.contains("= unless ") || t.ends_with("= unless")
                || t.contains("= case ") || t.ends_with("= case")
                || t.contains(" << if ") || t.ends_with(" << if")
                || t.contains(" << unless ") || t.ends_with(" << unless")
                || t.contains(" << case ") || t.ends_with(" << case")
                // `var = begin` / `var ||= begin` / `x || begin` inline begin/rescue/end block
                || t.ends_with("= begin") || t.ends_with("|| begin") || t.ends_with("&& begin")
                || t.contains("= begin ")
            );

            if is_def_opener {
                stack.push((i, indent, true));
            } else if (is_inner_construct || has_inline_conditional) && !stack.is_empty() {
                stack.push((i, indent, false));
            }

            if t == "end" || t.starts_with("end ") || t.starts_with("end.") {
                if let Some((_def_line, expected_indent, is_def)) = stack.pop() {
                    if is_def && indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`end` at indent {} does not match `def` at indent {}.",
                                indent, expected_indent
                            ),
                            range: TextRange::new(pos, pos + 3),
                            severity: Severity::Warning,
                        });
                    }
                    // If not is_def (inner construct), pop silently
                }
            }
        }

        diags
    }
}

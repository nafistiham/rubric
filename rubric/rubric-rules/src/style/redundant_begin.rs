use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantBegin;

/// Find the index of the `end` that closes the block opened at `begin_line`.
/// Uses simple nesting: counts openers/closers, returns the line index of
/// the matching `end`, or `None` if not found before EOF.
fn find_matching_end(lines: &[&str], begin_line: usize) -> Option<usize> {
    let mut depth: i32 = 1;
    let mut i = begin_line + 1;
    while i < lines.len() {
        let t = lines[i].trim();
        // Skip comments
        if t.starts_with('#') {
            i += 1;
            continue;
        }
        // Openers that push depth
        if is_block_opener(t) {
            depth += 1;
        }
        // `end` closes one level
        if is_end_token(t) {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn is_end_token(t: &str) -> bool {
    t == "end"
        || (t.starts_with("end")
            && t.len() > 3
            && {
                let b = t.as_bytes()[3];
                !b.is_ascii_alphanumeric() && b != b'_'
            })
}

fn is_block_opener(t: &str) -> bool {
    t == "begin"
        || t.starts_with("begin ")
        || t.ends_with(" begin")
        || t == "do"
        || t.ends_with(" do")
        || t.contains(" do |")
        || t.contains(" do|")
        || (t.starts_with("def ") || t == "def")
        || (t.starts_with("class ") || t == "class")
        || (t.starts_with("module ") || t == "module")
        || t.starts_with("if ")
        || t.starts_with("unless ")
        || t.starts_with("while ")
        || t.starts_with("until ")
        || t.starts_with("for ")
        || t.starts_with("case ")
        || t == "case"
}

/// Returns true when the `begin` at `begin_line` is the sole statement in the
/// method body — i.e., after its matching `end` there is no non-blank,
/// non-comment line before the method's closing `end`.
fn begin_spans_entire_method(lines: &[&str], begin_line: usize) -> bool {
    let end_idx = match find_matching_end(lines, begin_line) {
        Some(idx) => idx,
        None => return false,
    };
    // Check every line after the begin's `end` for real code.
    let mut i = end_idx + 1;
    while i < lines.len() {
        let t = lines[i].trim();
        if t.is_empty() || t.starts_with('#') {
            i += 1;
            continue;
        }
        // The only acceptable non-blank token after the begin's end is the
        // method's own closing `end`.
        if is_end_token(t) {
            return true;
        }
        // Any other real code means the begin does NOT span the entire body.
        return false;
    }
    false
}

impl Rule for RedundantBegin {
    fn name(&self) -> &'static str {
        "Style/RedundantBegin"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Detect `def` followed immediately by `begin` as first non-blank line,
        // AND only when the `begin...end` block spans the entire method body.
        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("def ") || trimmed == "def" {
                // Find the first non-blank line after `def`
                let mut j = i + 1;
                while j < n && lines[j].trim().is_empty() {
                    j += 1;
                }
                if j < n && lines[j].trim() == "begin" {
                    // Only flag when begin spans the full method body.
                    if begin_spans_entire_method(lines, j) {
                        let line_start = ctx.line_start_offsets[j];
                        let indent = lines[j].len() - lines[j].trim_start().len();
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Redundant `begin` block in method body.".into(),
                            range: TextRange::new(
                                line_start + indent as u32,
                                line_start + indent as u32 + 5,
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            i += 1;
        }

        diags
    }
}

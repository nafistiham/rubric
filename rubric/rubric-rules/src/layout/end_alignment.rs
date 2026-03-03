use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EndAlignment;

/// Returns true if `trimmed` is an endless method (`def foo = expr` / `def foo(x) = expr`).
/// Endless methods never have a matching `end` and should not be pushed onto the stack.
fn is_endless_method(trimmed: &str) -> bool {
    let def_pos = match trimmed.find("def ") {
        Some(p) if p <= 20 => p, // "def " near start of trimmed line
        _ => return false,
    };
    let after_def = &trimmed[def_pos + 4..]; // skip "def "
    let bytes = after_def.as_bytes();
    let n = bytes.len();
    let mut depth = 0i32;
    let mut i = 0;
    while i < n {
        match bytes[i] {
            b'(' => { depth += 1; }
            b')' => { depth -= 1; }
            // " = " at depth 0 (not "==" or "=>") indicates endless method
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

/// Returns true if `trimmed` is a one-liner like `class Foo; end` or `def foo; bar; end`.
/// These should not be pushed onto the stack because their `end` is on the same line.
fn is_one_liner(trimmed: &str) -> bool {
    trimmed.trim_end_matches(|c: char| c == ' ' || c == '\t').ends_with("; end")
}

impl Rule for EndAlignment {
    fn name(&self) -> &'static str {
        "Layout/EndAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Unified stack: (indent, check_alignment)
        // check_alignment=true  → block opener at line start (if/while/def/class/do/begin)
        // check_alignment=false → inline opener mid-line (= if / = begin / etc.)
        let mut stack: Vec<(usize, bool)> = Vec::new();

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            // Skip comments
            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Detect block/construct openers at the start of the trimmed line.
            // Exclude: one-liners (`class Foo; end`), endless methods (`def foo = expr`).
            let is_block_opener = !is_one_liner(trimmed) && !is_endless_method(trimmed) && (
                trimmed.starts_with("def ") || trimmed == "def"
                || trimmed.starts_with("private def ") || trimmed.starts_with("protected def ")
                || trimmed.starts_with("class ") || trimmed.starts_with("module ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed == "begin"
                || trimmed.starts_with("begin ") || trimmed.starts_with("case ")
                || trimmed == "do" || trimmed.ends_with(" do") || trimmed.contains(" do |") || trimmed.contains(" do|")
            );

            // Detect inline if/unless/case/begin assignments that open a block mid-line
            // Pattern: something = if condition  (or unless/case/begin, with any = variant: =, ||=, &&=, etc.)
            let is_inline_opener = !is_block_opener && (
                // Any assignment operator (=, ||=, &&=, +=, etc.) followed by if/unless/case
                (trimmed.contains("= if ") || trimmed.contains("= unless ") || trimmed.contains("= case "))
                || (trimmed.contains(" << if ") || trimmed.contains(" << unless ") || trimmed.contains(" << case "))
                || (trimmed.contains("(if ") || trimmed.contains("(unless ") || trimmed.contains("(case "))
                // `var = begin` / `var ||= begin` / `x || begin` — inline begin/rescue/end block
                || trimmed.ends_with("= begin") || trimmed.ends_with("|| begin") || trimmed.ends_with("&& begin")
                || trimmed.contains("= begin ")
            );

            if is_block_opener {
                stack.push((indent, true));
            } else if is_inline_opener {
                stack.push((indent, false));
            }

            // Detect end (including end.method chaining and end followed by other tokens)
            let is_end = trimmed == "end"
                || trimmed.starts_with("end ")
                || trimmed.starts_with("end.");

            if is_end {
                if let Some((expected_indent, check)) = stack.pop() {
                    if check && indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`end` indentation ({}) does not match its opener ({}).",
                                indent, expected_indent
                            ),
                            range: TextRange::new(
                                line_start + indent as u32,
                                line_start + indent as u32 + 3,
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

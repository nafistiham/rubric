use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

/// Returns true if the trimmed `def` line is a Ruby 3 endless method definition,
/// i.e. `def method_name(args) = expression`.  These are single-line and have no
/// multi-line body, so the blank line that follows is not "inside" the body.
///
/// Detection: after the `def` keyword, scan past the name and optional argument
/// list for a bare `=` that is NOT part of `==`, `=>`, `!=`, `<=`, or `>=`.
fn is_endless_method(trimmed: &str) -> bool {
    let bytes = trimmed.as_bytes();
    let len = bytes.len();
    // Must start with "def "
    if len < 5 || &bytes[..4] != b"def " {
        return false;
    }
    // Scan forward; once we see `=` that is not part of a compound operator,
    // and we are past the method-name/args region, it's an endless method.
    let mut j = 4usize;
    let mut paren_depth: i32 = 0;
    while j < len {
        let b = bytes[j];
        match b {
            b'(' => { paren_depth += 1; j += 1; }
            b')' => { paren_depth -= 1; j += 1; }
            b'=' if paren_depth == 0 => {
                // Check surrounding chars to rule out compound operators
                let prev = if j > 0 { bytes[j - 1] } else { 0 };
                let next = if j + 1 < len { bytes[j + 1] } else { 0 };
                if prev != b'!' && prev != b'<' && prev != b'>' && prev != b'='
                    && next != b'=' && next != b'>'
                {
                    // Bare `=` outside parens after the method name — endless method
                    return true;
                }
                j += 1;
            }
            _ => { j += 1; }
        }
    }
    false
}

pub struct EmptyLinesAroundMethodBody;

impl Rule for EmptyLinesAroundMethodBody {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundMethodBody"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Stack: true = def opener, false = other opener
        let mut opener_stack: Vec<bool> = Vec::new();

        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();

            if trimmed.starts_with("def ") || trimmed == "def" {
                // Skip single-line method definitions: `def foo; end` or `def foo; body; end`
                // These close on the same line — no multi-line body to check.
                if trimmed.ends_with("; end") {
                    i += 1;
                    continue;
                }
                // Skip Ruby 3 endless method definitions: `def foo = expression`
                // Detect the ` = ` assignment form (not `==`, `!=`, `<=`, `>=`, `=>`).
                if is_endless_method(trimmed) {
                    i += 1;
                    continue;
                }
                // Check if the next line is blank
                if i + 1 < n && lines[i + 1].trim().is_empty() {
                    let line_start = ctx.line_start_offsets[i + 1];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra empty line after method body start.".into(),
                        range: TextRange::new(line_start, line_start + lines[i + 1].len() as u32),
                        severity: Severity::Warning,
                    });
                }
                opener_stack.push(true);
            } else if trimmed.starts_with("class ") || trimmed.starts_with("module ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed.starts_with("begin")
                || trimmed == "do" || trimmed.ends_with(" do") {
                opener_stack.push(false);
            } else if trimmed == "end" {
                if let Some(is_def) = opener_stack.pop() {
                    if is_def {
                        // Check if line before `end` is blank
                        if i > 0 && lines[i - 1].trim().is_empty() {
                            let line_start = ctx.line_start_offsets[i - 1];
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Extra empty line before method body end.".into(),
                                range: TextRange::new(
                                    line_start,
                                    line_start + lines[i - 1].len() as u32,
                                ),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
            i += 1;
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct BitwiseOperatorInConditional;

/// Keywords that introduce conditionals, followed by a space and a condition.
const CONDITIONAL_PREFIXES: &[&str] = &["if ", "elsif ", "unless ", "while ", "until "];

impl Rule for BitwiseOperatorInConditional {
    fn name(&self) -> &'static str {
        "Style/BitwiseOperatorInConditional"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Check if the line starts with a conditional keyword
            let condition_part = CONDITIONAL_PREFIXES
                .iter()
                .find_map(|prefix| trimmed.strip_prefix(prefix));

            let condition = match condition_part {
                Some(c) => c,
                None => continue,
            };

            // Strip inline comment from the condition
            let scan_end = comment_start(condition).unwrap_or(condition.len());
            let condition = &condition[..scan_end];

            if has_single_bitwise_operator(condition) {
                let line_start = ctx.line_start_offsets[i] as usize;
                let start = line_start as u32;
                let end = start + line.len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Avoid using a bitwise operator in a conditional; use || or && instead.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Returns true when `condition` contains a bare `&` or `|` that is not part
/// of `&&`, `||`, `&.`, `&=`, or `|=`.
fn has_single_bitwise_operator(condition: &str) -> bool {
    let bytes = condition.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let b = bytes[i];

        if b == b'&' {
            let prev_is_amp = i > 0 && bytes[i - 1] == b'&';
            let next_is_amp = i + 1 < len && bytes[i + 1] == b'&';
            let next_is_dot = i + 1 < len && bytes[i + 1] == b'.';
            let next_is_eq = i + 1 < len && bytes[i + 1] == b'=';

            if !prev_is_amp && !next_is_amp && !next_is_dot && !next_is_eq {
                return true;
            }
        } else if b == b'|' {
            let prev_is_pipe = i > 0 && bytes[i - 1] == b'|';
            let next_is_pipe = i + 1 < len && bytes[i + 1] == b'|';
            let next_is_eq = i + 1 < len && bytes[i + 1] == b'=';

            if !prev_is_pipe && !next_is_pipe && !next_is_eq {
                return true;
            }
        }

        i += 1;
    }

    false
}

/// Returns the index of the comment character `#` on the line, ignoring
/// `#` that appear inside string literals.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
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
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        i += 1;
    }
    None
}

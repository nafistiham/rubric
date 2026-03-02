use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineMethodCallIndentation;

/// Returns the byte index of the first `#` that is not inside a string literal,
/// or `None` if no such `#` exists.
fn find_inline_comment_start(line: &str) -> Option<usize> {
    let mut in_double = false;
    let mut in_single = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut byte_offset = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Handle escape sequences inside strings
        if (in_double || in_single) && ch == '\\' && i + 1 < chars.len() {
            // Skip the escaped character
            byte_offset += ch.len_utf8();
            i += 1;
            byte_offset += chars[i].len_utf8();
            i += 1;
            continue;
        }

        if ch == '"' && !in_single {
            in_double = !in_double;
        } else if ch == '\'' && !in_double {
            in_single = !in_single;
        } else if ch == '#' && !in_double && !in_single {
            return Some(byte_offset);
        }

        byte_offset += ch.len_utf8();
        i += 1;
    }

    None
}

impl Rule for MultilineMethodCallIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodCallIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_end();

            // Skip pure comment lines (the non-whitespace content starts with `#`)
            if trimmed.trim_start().starts_with('#') {
                continue;
            }

            // For mixed lines, strip the inline comment before checking for trailing dot
            let code_part = match find_inline_comment_start(trimmed) {
                Some(idx) => trimmed[..idx].trim_end(),
                None => trimmed,
            };

            // A code line whose code part ends with `.` indicates a chained call
            // continuation (trailing dot style)
            if code_part.ends_with('.') {
                let line_start = ctx.line_start_offsets[i] as usize;
                let dot_offset = line_start + code_part.len() - 1;
                let pos = dot_offset as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Chained method call continuation detected — ensure proper indentation.".into(),
                    range: TextRange::new(pos, pos + 1),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

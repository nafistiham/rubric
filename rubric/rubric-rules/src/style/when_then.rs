use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct WhenThen;

impl Rule for WhenThen {
    fn name(&self) -> &'static str {
        "Style/WhenThen"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Must start with `when `
            if !trimmed.starts_with("when ") {
                continue;
            }

            // Strip inline comment to get the actual code part
            let code_part = strip_inline_comment(trimmed);
            let code_trimmed = code_part.trim_end();

            // The line must end with ` then` (word boundary) to be flagged.
            // This means `then` is at the end of the line (multiline form).
            // If something follows `then` on the same line, it's inline and OK.
            if !ends_with_word(code_trimmed, "then") {
                continue;
            }

            // Confirm there is content between `when` and `then`
            // i.e. the line is `when <expr> then` not just `when then`
            // (though `when then` is also flagged — it's still a misuse)
            let line_start = ctx.line_start_offsets[i] as usize;
            let indent = line.len() - trimmed.len();
            let start = (line_start + indent) as u32;
            let end = (line_start + line.trim_end().len()) as u32;

            diags.push(Diagnostic {
                rule: self.name(),
                message: "Do not use `then` for multiline `when`.".into(),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

/// Strip trailing inline comment from a line of code.
/// Handles simple `# comment` but not `#` inside strings.
fn strip_inline_comment(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut in_string: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match in_string {
            Some(delim) => {
                if b == b'\\' {
                    i += 2;
                    continue;
                }
                if b == delim {
                    in_string = None;
                }
            }
            None => match b {
                b'"' | b'\'' => in_string = Some(b),
                b'#' => return &s[..i],
                _ => {}
            },
        }
        i += 1;
    }
    s
}

/// Returns true if `s` ends with the word `word` (preceded by a non-word char or start).
fn ends_with_word(s: &str, word: &str) -> bool {
    if !s.ends_with(word) {
        return false;
    }
    let before = s.len() - word.len();
    if before == 0 {
        return true;
    }
    let prev_byte = s.as_bytes()[before - 1];
    // The character before `word` must be a non-word character (space, etc.)
    !prev_byte.is_ascii_alphanumeric() && prev_byte != b'_'
}

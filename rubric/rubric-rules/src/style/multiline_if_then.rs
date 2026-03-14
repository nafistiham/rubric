use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineIfThen;

impl Rule for MultilineIfThen {
    fn name(&self) -> &'static str {
        "Style/MultilineIfThen"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Must start with `if ` or `unless `
            if !trimmed.starts_with("if ") && !trimmed.starts_with("unless ") {
                continue;
            }

            // Skip one-liners: if the line also contains `end` (on same line) it's fine
            if trimmed.contains(" end") || trimmed.ends_with(";") {
                continue;
            }

            // Check if line ends with `then` (after stripping inline comment)
            let code_part = strip_inline_comment(trimmed);
            let code_trimmed = code_part.trim_end();

            if !code_trimmed.ends_with(" then") && !code_trimmed.ends_with("\tthen") {
                continue;
            }

            // It must be a multi-line construct: the next non-blank line should not be `end`
            // Simple check: if the line has a semicolon it's a one-liner — already handled above
            let line_start = ctx.line_start_offsets[i];
            let line_end = line_start + line.len() as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Do not use `then` for multi-line `if`/`unless`.".into(),
                range: TextRange::new(line_start, line_end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

/// Strip inline comment (`# ...`) from a code line, returning the code portion.
/// Respects strings so `#` inside quotes is not treated as a comment.
fn strip_inline_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut in_string: Option<u8> = None;
    let mut j = 0;
    while j < n {
        let b = bytes[j];
        if let Some(delim) = in_string {
            if b == b'\\' {
                j += 2;
                continue;
            }
            if b == delim {
                in_string = None;
            }
        } else {
            match b {
                b'"' | b'\'' | b'`' => in_string = Some(b),
                b'#' => return &line[..j],
                _ => {}
            }
        }
        j += 1;
    }
    line
}

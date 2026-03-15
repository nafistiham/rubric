use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AsciiComments;

/// Find the byte offset of the `#` comment marker on a line, skipping `#`
/// that appear inside string literals. Returns `None` if no real comment exists.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut interp_depth: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(b'"') if bytes[i..].starts_with(b"#{") => {
                interp_depth += 1;
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d && interp_depth == 0 => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        if interp_depth > 0 && bytes[i] == b'}' {
            interp_depth -= 1;
        }
        i += 1;
    }
    None
}

impl Rule for AsciiComments {
    fn name(&self) -> &'static str {
        "Style/AsciiComments"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            let Some(comment_pos) = comment_start(line) else {
                continue;
            };

            let comment_text = &line[comment_pos..];
            let line_start = ctx.line_start_offsets[line_idx] as usize;

            // Check if any byte in the comment is non-ASCII (> 127)
            let has_non_ascii = comment_text.bytes().any(|b| b > 127);
            if has_non_ascii {
                let start = (line_start + comment_pos) as u32;
                let end = (line_start + line.len()) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use only ascii symbols in comments.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

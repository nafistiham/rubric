use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct StabbyLambdaParentheses;

const MESSAGE: &str = "Remove the space before the lambda parameters.";

/// Returns `true` if `pos` in `bytes` is inside a string literal (`"` or `'`).
/// Stops scanning at an unquoted `#` (comment start).
fn in_string_at(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos && i < bytes.len() {
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
            None if bytes[i] == b'#' => return false,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

impl Rule for StabbyLambdaParentheses {
    fn name(&self) -> &'static str {
        "Style/StabbyLambdaParentheses"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // The pattern we search for: `-> (`  (arrow, space, open-paren)
        const PATTERN: &[u8] = b"-> (";

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            let mut search = 0usize;
            while search + PATTERN.len() <= bytes.len() {
                let found = bytes[search..]
                    .windows(PATTERN.len())
                    .position(|w| w == PATTERN);

                let rel = match found {
                    Some(r) => r,
                    None => break,
                };
                let abs = search + rel;

                // Skip if inside a string literal
                if in_string_at(bytes, abs) {
                    search = abs + PATTERN.len();
                    continue;
                }

                // The offending space is at position abs + 2 (between `->` and `(`)
                let space_pos = abs + 2;
                let start = (line_start + space_pos) as u32;
                let end = start + 1; // one space character
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: MESSAGE.into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });

                search = abs + PATTERN.len();
            }
        }

        diags
    }
}

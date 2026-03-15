use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RescueType;

const MESSAGE: &str = "Rescue cannot rescue non-exception class.";

/// Returns true if `word` is a literal value that is clearly not an exception class.
fn is_non_exception_literal(word: &str) -> bool {
    matches!(word, "nil" | "true" | "false")
        || word.chars().all(|c| c.is_ascii_digit())
}

/// Returns number of leading whitespace bytes (space or tab).
fn skip_whitespace(s: &[u8]) -> usize {
    s.iter().take_while(|&&c| c == b' ' || c == b'\t').count()
}

/// Read an alphanumeric/underscore token from the start of `s`.
/// Returns the token as `&str` and the number of bytes consumed, or `None`.
fn read_token(s: &[u8]) -> Option<(&str, usize)> {
    if s.is_empty() {
        return None;
    }
    let first = s[0];
    if !first.is_ascii_alphanumeric() && first != b'_' {
        return None;
    }
    let len = s
        .iter()
        .take_while(|&&c| c.is_ascii_alphanumeric() || c == b'_')
        .count();
    let tok = std::str::from_utf8(&s[..len]).ok()?;
    Some((tok, len))
}

impl Rule for RescueType {
    fn name(&self) -> &'static str {
        "Lint/RescueType"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip pure comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Quick pre-screen: must start with `rescue`
            if !trimmed.starts_with("rescue") {
                continue;
            }

            let bytes = trimmed.as_bytes();
            let rescue_len = b"rescue".len();

            // Must be `rescue` followed by whitespace (not `rescue_something`)
            if bytes.len() <= rescue_len {
                continue;
            }
            let after_rescue = bytes[rescue_len];
            if after_rescue != b' ' && after_rescue != b'\t' {
                continue;
            }

            // Skip whitespace after `rescue`
            let rest = &bytes[rescue_len..];
            let ws = skip_whitespace(rest);
            let token_bytes = &bytes[rescue_len + ws..];

            // Read the first token after `rescue`
            if let Some((tok, _)) = read_token(token_bytes) {
                if is_non_exception_literal(tok) {
                    let indent = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let start = (line_start + indent) as u32;
                    let span_len = rescue_len + ws + tok.len();
                    let end = start + span_len as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: MESSAGE.into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantSafeNavigation;

/// Find all `&.` positions in `code` (the non-comment portion of a line)
/// and for each, check whether the immediately preceding non-whitespace
/// character indicates a literal receiver that can never be nil.
///
/// Literal indicators:
/// - `"` or `'`  — end of a string literal
/// - `]`         — end of an array literal
/// - `}`         — end of a hash literal
/// - `0-9`       — end of a numeric literal
fn scan_line_for_violations(
    code: &str,
    line_offset: u32,
) -> Vec<(u32, u32)> {
    let bytes = code.as_bytes();
    let mut violations = Vec::new();
    let mut i = 0;

    while i + 1 < bytes.len() {
        if bytes[i] == b'&' && bytes[i + 1] == b'.' {
            // Look backwards for the last non-whitespace character before `&.`
            if i > 0 {
                let mut j = i - 1;
                // Skip whitespace
                while j > 0 && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j -= 1;
                }
                let preceding = bytes[j];
                // Only flag when preceding character unambiguously indicates a
                // literal that can never be nil. `]` and `}` are excluded because
                // they can end a hash/array *access* (e.g. `x[k]&.m`) which CAN
                // return nil, not just a literal.
                let is_literal_receiver = matches!(
                    preceding,
                    b'"' | b'\'' | b'0'..=b'9'
                );
                if is_literal_receiver {
                    let start = line_offset + i as u32;
                    let end = line_offset + (i + 2) as u32;
                    violations.push((start, end));
                }
            }
            i += 2;
            continue;
        }
        i += 1;
    }

    violations
}

/// Returns the byte index where a comment begins on `line`, accounting for
/// string literals so that `#` inside strings is not treated as a comment.
fn find_comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1; // skip closing quote
                }
            }
            b'#' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

impl Rule for RedundantSafeNavigation {
    fn name(&self) -> &'static str {
        "Lint/RedundantSafeNavigation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            let code_end = find_comment_start(line).unwrap_or(line.len());
            let code = &line[..code_end];
            let line_offset = ctx.line_start_offsets[line_idx];

            for (start, end) in scan_line_for_violations(code, line_offset) {
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Redundant safe navigation detected.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

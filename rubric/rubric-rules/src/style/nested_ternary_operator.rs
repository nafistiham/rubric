use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NestedTernaryOperator;

/// Count ternary-style `?` characters on a line.
///
/// A `?` is considered a ternary operator (not a method predicate suffix) when
/// it is preceded by a whitespace character, `)`, `]`, `'`, or `"`. Method
/// predicate `?` (like `foo?`) is immediately appended to word characters with
/// no space and does not count.
///
/// Also requires that after the `?` there is a `:` somewhere on the line that
/// is not part of a `=>` (hash rocket) or `::` (constant separator). If a `?`
/// has no subsequent plain `:`, it is not a ternary operator.
fn count_ternary_questions(line: &str) -> usize {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut count = 0usize;

    let mut i = 0usize;
    while i < len {
        if bytes[i] == b'?' {
            // Check that the char preceding `?` indicates ternary position.
            let is_ternary_position = if i == 0 {
                false
            } else {
                matches!(
                    bytes[i - 1],
                    b' ' | b'\t' | b')' | b']' | b'\'' | b'"'
                )
            };

            if is_ternary_position {
                // Verify there is a bare `:` after this `?` on the same line
                // (not `=>`, `::`, or symbol literal `:word`).
                if has_ternary_colon(bytes, i + 1) {
                    count += 1;
                }
            }
        }
        i += 1;
    }

    count
}

/// Return true if there is a ternary `:` in `bytes[from..]`.
///
/// A ternary `:` is a `:` that:
/// - Is not part of `=>` (preceded by `=`)
/// - Is not part of `::` (followed by `:`)
/// - Is not part of `::` (preceded by `:`)
/// - Has a space or `)` before or after it (i.e., not a symbol literal like `:foo`)
fn has_ternary_colon(bytes: &[u8], from: usize) -> bool {
    let len = bytes.len();
    let mut i = from;
    while i < len {
        if bytes[i] == b':' {
            // Not a hash rocket: `=>`
            let prev_is_eq = i > 0 && bytes[i - 1] == b'=';
            // Not `::` double colon
            let next_is_colon = i + 1 < len && bytes[i + 1] == b':';
            let prev_is_colon = i > 0 && bytes[i - 1] == b':';

            if !prev_is_eq && !next_is_colon && !prev_is_colon {
                // It is a bare `:` — treat as ternary colon.
                return true;
            }
        }
        i += 1;
    }
    false
}

impl Rule for NestedTernaryOperator {
    fn name(&self) -> &'static str {
        "Style/NestedTernaryOperator"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines.
            if trimmed.starts_with('#') {
                continue;
            }

            let count = count_ternary_questions(line);
            if count >= 2 {
                let line_start = ctx.line_start_offsets[i];
                // Report at the beginning of the line (the first character).
                let indent = (line.len() - trimmed.len()) as u32;
                let start = line_start + indent;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Ternary operators must not be nested. Prefer if or case expressions."
                        .into(),
                    range: TextRange::new(start, start + 1),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

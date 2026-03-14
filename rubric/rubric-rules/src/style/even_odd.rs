use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EvenOdd;

/// Patterns that indicate a manual even/odd check and the suggested replacement.
const PATTERNS: &[(&str, &str)] = &[
    ("% 2 == 0", "Use `Integer#even?` instead of `integer % 2 == 0`."),
    ("% 2 != 0", "Use `Integer#odd?` instead of `integer % 2 != 0`."),
    ("% 2 == 1", "Use `Integer#odd?` instead of `integer % 2 == 1`."),
    ("% 2 != 1", "Use `Integer#even?` instead of `integer % 2 != 1`."),
];

impl Rule for EvenOdd {
    fn name(&self) -> &'static str {
        "Style/EvenOdd"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip pure comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Strip string literals from the line before pattern matching to avoid
            // false positives on string contents.
            let code_only = strip_strings(line);

            let line_start = ctx.line_start_offsets[i] as usize;

            for (pattern, message) in PATTERNS {
                if let Some(col) = code_only.find(pattern) {
                    let start = (line_start + col) as u32;
                    let end = (line_start + col + pattern.len()) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: (*message).into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                    // Only flag first matching pattern per line to avoid duplicate reports
                    break;
                }
            }
        }

        diags
    }
}

/// Replace the content of single- and double-quoted string literals on a line with spaces
/// so that pattern matching does not fire on string contents.
/// This is a single-line scanner and does not handle multi-line strings.
fn strip_strings(line: &str) -> String {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut result = Vec::with_capacity(n);
    let mut i = 0;
    let mut in_string: Option<u8> = None;

    while i < n {
        let b = bytes[i];
        match in_string {
            Some(delim) => {
                if b == b'\\' {
                    // Escape sequence: replace both chars with spaces
                    result.push(b' ');
                    result.push(b' ');
                    i += 2;
                    continue;
                } else if b == delim {
                    in_string = None;
                    result.push(b' '); // replace closing quote with space
                } else {
                    result.push(b' '); // replace string body with space
                }
            }
            None => {
                if b == b'"' || b == b'\'' {
                    in_string = Some(b);
                    result.push(b' '); // replace opening quote with space
                } else if b == b'#' {
                    // Inline comment: stop processing
                    break;
                } else {
                    result.push(b);
                }
            }
        }
        i += 1;
    }

    String::from_utf8(result).unwrap_or_default()
}

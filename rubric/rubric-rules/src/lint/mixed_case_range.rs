use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MixedCaseRange;

impl Rule for MixedCaseRange {
    fn name(&self) -> &'static str {
        "Lint/MixedCaseRange"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip pure comment lines
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i];

            // Scan for regex literals on this line and check for mixed-case ranges inside them.
            let violations = find_mixed_case_ranges_in_line(line);
            for col in violations {
                let start = line_start + col as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message:
                        "Avoid using a range of characters that mixes upper and lower case."
                            .into(),
                    range: TextRange::new(start, start + 3), // covers `X-Y` (3 chars)
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Find all byte column positions of mixed-case character ranges within regex literals on a line.
/// Returns column positions of the first character of the problematic range (e.g., `A` in `A-z`).
fn find_mixed_case_ranges_in_line(line: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut results = Vec::new();

    let mut i = 0;
    while i < n {
        let b = bytes[i];

        // Skip string literals: single-quoted and double-quoted strings
        if b == b'\'' || b == b'"' {
            let delim = b;
            i += 1;
            while i < n {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == delim {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Skip inline comments
        if b == b'#' {
            break;
        }

        // Detect regex literal: `/` not preceded by common operators that would make it division.
        // We use a simple heuristic: `/` starts a regex if the previous non-space character
        // is one of: start-of-line, `=`, `(`, `,`, `[`, `!`, `&`, `|`, `{`, `;`, `~`, `%`
        if b == b'/' {
            // Scan forward to find the end of the regex, collecting char classes.
            let regex_start = i;
            i += 1;
            let mut in_char_class = false;
            let mut char_class_start = 0;

            while i < n {
                let rb = bytes[i];
                if rb == b'\\' {
                    i += 2;
                    continue;
                }
                if in_char_class {
                    if rb == b']' {
                        in_char_class = false;
                    } else {
                        // Check for a range: `X-Y` where one is upper, one is lower
                        if i + 2 < n && bytes[i + 1] == b'-' {
                            let left = rb;
                            let right = bytes[i + 2];
                            if is_mixed_case_range(left, right) {
                                // Column of the left char within the line
                                results.push(i);
                            }
                        }
                    }
                } else if rb == b'[' {
                    in_char_class = true;
                    char_class_start = i;
                } else if rb == b'/' {
                    // End of regex
                    i += 1;
                    break;
                }
                i += 1;
            }
            let _ = (regex_start, char_class_start);
            continue;
        }

        i += 1;
    }

    results
}

/// Return true if `left` and `right` form a mixed upper/lower case range.
/// A mixed range has one endpoint as ASCII uppercase (A-Z) and the other as ASCII lowercase (a-z).
fn is_mixed_case_range(left: u8, right: u8) -> bool {
    (left.is_ascii_uppercase() && right.is_ascii_lowercase())
        || (left.is_ascii_lowercase() && right.is_ascii_uppercase())
}

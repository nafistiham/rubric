use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantCapitalW;

const MESSAGE: &str =
    "Use %w or %W only when interpolation is needed; prefer %w for arrays without interpolation.";

/// Returns the closing delimiter for the given opening delimiter byte.
fn closing_delim(open: u8) -> u8 {
    match open {
        b'[' => b']',
        b'(' => b')',
        b'{' => b'}',
        other => other,
    }
}

impl Rule for RedundantCapitalW {
    fn name(&self) -> &'static str {
        "Style/RedundantCapitalW"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;
        let bytes = src.as_bytes();
        let len = bytes.len();

        let mut i = 0usize;
        while i + 3 <= len {
            // Look for `%W` followed by a delimiter
            if bytes[i] == b'%' && bytes[i + 1] == b'W' {
                let delim_byte = bytes[i + 2];
                // Only handle recognised delimiters
                if matches!(delim_byte, b'[' | b'(' | b'{' | b'/' | b'|' | b'!') {
                    let close = closing_delim(delim_byte);
                    let content_start = i + 3;

                    // Scan for the closing delimiter, tracking nesting
                    let mut j = content_start;
                    let mut depth = 1u32;
                    while j < len {
                        if bytes[j] == b'\\' {
                            j += 2;
                            continue;
                        }
                        if bytes[j] == delim_byte && delim_byte != close {
                            depth += 1;
                        } else if bytes[j] == close {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        j += 1;
                    }

                    let content = &src[content_start..j];

                    // Flag only if no interpolation present
                    if !content.contains("#{") {
                        // Find the line number for this position to get line_start_offset
                        let start = i as u32;
                        let end = (j + 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: MESSAGE.into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                    }

                    i = j + 1;
                    continue;
                }
            }
            i += 1;
        }

        diags
    }
}

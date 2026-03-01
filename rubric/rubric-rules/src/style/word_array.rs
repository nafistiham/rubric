use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct WordArray;

impl Rule for WordArray {
    fn name(&self) -> &'static str {
        "Style/WordArray"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;

            while j < len {
                // Look for `[` to start an array literal
                if bytes[j] != b'[' {
                    j += 1;
                    continue;
                }

                let array_start = j;
                j += 1;

                // Skip whitespace
                while j < len && bytes[j] == b' ' { j += 1; }

                // Check if first element is a double-quoted string
                if j >= len || bytes[j] != b'"' {
                    j = array_start + 1;
                    continue;
                }

                // Try to parse all elements as simple word strings
                let mut k = j;
                let mut word_count = 0;
                let mut valid = true;

                while k < len {
                    // Skip whitespace
                    while k < len && bytes[k] == b' ' { k += 1; }

                    if k >= len { valid = false; break; }

                    if bytes[k] == b']' {
                        break;
                    }

                    // Expect `"word"` — only word chars inside
                    if bytes[k] != b'"' {
                        valid = false;
                        break;
                    }
                    k += 1;

                    // Read string content — only word chars allowed
                    let str_start = k;
                    while k < len && bytes[k] != b'"' && bytes[k] != b'\n' {
                        if !bytes[k].is_ascii_alphanumeric() && bytes[k] != b'_' {
                            valid = false;
                            break;
                        }
                        k += 1;
                    }

                    if !valid { break; }
                    if k == str_start || k >= len || bytes[k] != b'"' {
                        // Empty string or no closing quote
                        valid = false;
                        break;
                    }
                    k += 1; // skip closing `"`
                    word_count += 1;

                    // Skip whitespace
                    while k < len && bytes[k] == b' ' { k += 1; }

                    if k >= len { valid = false; break; }
                    if bytes[k] == b']' {
                        break;
                    } else if bytes[k] == b',' {
                        k += 1;
                    } else {
                        valid = false;
                        break;
                    }
                }

                if valid && word_count >= 2 && k < len && bytes[k] == b']' {
                    let start = (line_start + array_start) as u32;
                    let end = (line_start + k + 1) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `%w[]` for arrays of strings.".into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                    j = k + 1;
                } else {
                    j = array_start + 1;
                }
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FloatOutOfRange;

impl Rule for FloatOutOfRange {
    fn name(&self) -> &'static str {
        "Lint/FloatOutOfRange"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;
            // Track whether we are inside a string literal (single or double quoted).
            // We use a simple approach: toggle on unescaped quote characters.
            let mut in_string = false;
            let mut string_char = b'"';

            while j < len {
                let ch = bytes[j];

                // Handle string boundaries (single and double quotes).
                if !in_string && (ch == b'"' || ch == b'\'') {
                    in_string = true;
                    string_char = ch;
                    j += 1;
                    continue;
                }
                if in_string {
                    if ch == b'\\' {
                        // Skip escaped character
                        j += 2;
                        continue;
                    }
                    if ch == string_char {
                        in_string = false;
                    }
                    j += 1;
                    continue;
                }

                // Outside a string: look for digit sequences that could be float literals.
                if ch.is_ascii_digit() {
                    // Check that this digit is not preceded by a hex-context character
                    // (i.e. the digit starts a fresh token, not inside a word/identifier).
                    // A digit that is immediately preceded by a letter or digit is part of
                    // an identifier/word, not a standalone numeric literal.
                    if j > 0 {
                        let prev = bytes[j - 1];
                        if prev.is_ascii_alphanumeric() || prev == b'_' {
                            j += 1;
                            continue;
                        }
                    }

                    let lit_start = j;
                    // Read integer part: digits (and underscores for Ruby numeric separators)
                    while j < len
                        && (bytes[j].is_ascii_digit() || bytes[j] == b'_')
                    {
                        j += 1;
                    }
                    // Optional fractional part
                    if j < len && bytes[j] == b'.' {
                        j += 1;
                        while j < len && (bytes[j].is_ascii_digit() || bytes[j] == b'_') {
                            j += 1;
                        }
                    }
                    // Check for `e` or `E` exponent
                    if j < len && (bytes[j] == b'e' || bytes[j] == b'E') {
                        // The character after 'e' must be a digit or sign for this to be
                        // scientific notation. A bare `e` followed by a letter is not a float.
                        let e_pos = j;
                        j += 1;
                        // Optional sign
                        let mut has_sign = false;
                        if j < len && (bytes[j] == b'+' || bytes[j] == b'-') {
                            has_sign = true;
                            j += 1;
                        }
                        let digit_start = j;
                        while j < len && bytes[j].is_ascii_digit() {
                            j += 1;
                        }
                        let exp_digits = j - digit_start;

                        // Validate: must have at least one exponent digit, and the character
                        // immediately after the exponent must NOT be an alphanumeric character
                        // (which would indicate this `e` is inside a hex string / identifier).
                        let followed_by_alnum = j < len
                            && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_');

                        if exp_digits == 0 || followed_by_alnum {
                            // Not a real float literal — `e` is part of an identifier/hex token.
                            // Do not flag; continue scanning from just after the `e`.
                            j = e_pos + 1;
                            if has_sign {
                                // back up before sign too if we advanced past it
                                j = e_pos + 1;
                            }
                            continue;
                        }

                        // Now actually parse the literal to determine if it overflows.
                        // Collect the full literal text.
                        let lit_text = &line[lit_start..j];
                        // Strip underscores (Ruby allows 1_000_000.0e2)
                        let clean: String =
                            lit_text.chars().filter(|&c| c != '_').collect();
                        let overflows = clean.parse::<f64>().map_or(false, |v| v.is_infinite());
                        let underflows_to_zero = clean
                            .parse::<f64>()
                            .map_or(false, |v| v == 0.0 && clean != "0" && clean != "0.0");

                        if overflows || underflows_to_zero {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            let msg = if overflows {
                                "Float literal with large exponent will overflow to Infinity."
                            } else {
                                "Float literal with tiny exponent will underflow to 0.0."
                            };
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: msg.into(),
                                range: TextRange::new(
                                    (line_start + lit_start) as u32,
                                    (line_start + j) as u32,
                                ),
                                severity: Severity::Warning,
                            });
                        }
                        continue;
                    }
                    // No exponent — not a float out of range candidate.
                    continue;
                }

                j += 1;
            }
        }

        diags
    }
}

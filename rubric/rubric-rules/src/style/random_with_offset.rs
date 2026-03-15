use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RandomWithOffset;

/// Returns true if the byte position is inside a string literal or comment on the line.
/// This is a conservative line-level check: find the first `#` that is not inside
/// a string, and skip anything after it.
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
                        i += 1; // skip escaped char
                    }
                    i += 1;
                }
                // skip closing quote if present
                if i < bytes.len() {
                    i += 1;
                }
            }
            b'#' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

impl Rule for RandomWithOffset {
    fn name(&self) -> &'static str {
        "Style/RandomWithOffset"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            // Determine the portion of the line that is code (exclude comments).
            let code_end = find_comment_start(line).unwrap_or(line.len());
            let code = &line[..code_end];

            let bytes = code.as_bytes();
            let mut i = 0;

            while i < bytes.len() {
                // Look for `rand(`
                if bytes[i..].starts_with(b"rand(") {
                    // Verify word boundary before `rand`
                    let before_ok = i == 0 || {
                        let b = bytes[i - 1];
                        !b.is_ascii_alphanumeric() && b != b'_'
                    };

                    if before_ok {
                        let rand_start = i;
                        let after_rand = i + "rand(".len();

                        // Collect digits after `rand(`
                        let mut j = after_rand;
                        while j < bytes.len() && bytes[j].is_ascii_digit() {
                            j += 1;
                        }

                        // Must have at least one digit and be followed by `)`
                        if j > after_rand && j < bytes.len() && bytes[j] == b')' {
                            let close_paren = j;
                            j += 1; // skip `)`

                            // Skip optional whitespace
                            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                                j += 1;
                            }

                            // Must be followed by `+`
                            if j < bytes.len() && bytes[j] == b'+' {
                                j += 1;

                                // Skip optional whitespace
                                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                                    j += 1;
                                }

                                // Must be followed by digits
                                let digit_start = j;
                                while j < bytes.len() && bytes[j].is_ascii_digit() {
                                    j += 1;
                                }

                                if j > digit_start {
                                    let expr_end = j;
                                    let line_offset = ctx.line_start_offsets[line_idx];
                                    diags.push(Diagnostic {
                                        rule: self.name(),
                                        message: "Prefer rand(a..b) over rand(b) + a.".into(),
                                        range: TextRange::new(
                                            line_offset + rand_start as u32,
                                            line_offset + expr_end as u32,
                                        ),
                                        severity: Severity::Warning,
                                    });
                                    i = expr_end;
                                    continue;
                                }
                            }
                            // Not a matching pattern — advance past close paren
                            i = close_paren + 1;
                            continue;
                        }
                    }
                }
                i += 1;
            }
        }

        diags
    }
}

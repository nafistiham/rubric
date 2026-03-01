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

            // Detect float literals with large exponents: \d+e\d{3,} or \d+\.\d+e\d{3,}
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;

            while j < len {
                if bytes[j].is_ascii_digit() {
                    let lit_start = j;
                    // Read digits and optional dot
                    while j < len && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
                        j += 1;
                    }
                    // Check for `e` or `E`
                    if j < len && (bytes[j] == b'e' || bytes[j] == b'E') {
                        j += 1;
                        // Optional sign
                        if j < len && (bytes[j] == b'+' || bytes[j] == b'-') {
                            j += 1;
                        }
                        let digit_start = j;
                        while j < len && bytes[j].is_ascii_digit() {
                            j += 1;
                        }
                        let exp_digits = j - digit_start;
                        // If exponent has 3+ digits (>= 100), it's likely overflow
                        if exp_digits >= 3 {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Float literal with large exponent will overflow to Infinity.".into(),
                                range: TextRange::new((line_start + lit_start) as u32, (line_start + j) as u32),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    continue;
                }
                j += 1;
            }
        }

        diags
    }
}

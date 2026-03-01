use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct PercentLiteralDelimiters;

impl Rule for PercentLiteralDelimiters {
    fn name(&self) -> &'static str {
        "Style/PercentLiteralDelimiters"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;
        let bytes = src.as_bytes();
        let n = bytes.len();
        let mut i = 0;

        while i < n {
            // Look for %w( %i( %W( %I(
            if bytes[i] == b'%' && i + 2 < n {
                let kind = bytes[i + 1];
                if matches!(kind, b'w' | b'i' | b'W' | b'I') && bytes[i + 2] == b'(' {
                    // Find the matching `)`
                    let open_pos = i + 2;
                    let mut depth = 1usize;
                    let mut close_pos = open_pos + 1;
                    while close_pos < n && depth > 0 {
                        if bytes[close_pos] == b'(' { depth += 1; }
                        else if bytes[close_pos] == b')' { depth -= 1; }
                        if depth > 0 { close_pos += 1; }
                    }

                    // If no matching `)` was found, skip this occurrence
                    if depth > 0 {
                        i += 1;
                        continue;
                    }

                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Use `[]` delimiters instead of `()` for `%{}` literals.",
                            kind as char
                        ),
                        range: TextRange::new(i as u32, (close_pos + 1) as u32),
                        severity: Severity::Warning,
                    });
                    i = close_pos + 1;
                    continue;
                }
            }
            i += 1;
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // We need to replace `(` at start+2 with `[` and `)` at end-1 with `]`
        // But we only have the range, not the actual chars. Return None.
        let _ = diag;
        None
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SymbolProc;

impl Rule for SymbolProc {
    fn name(&self) -> &'static str {
        "Style/SymbolProc"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Look for pattern: `{ |word| word.method }` on a single line
            // Using a manual scan: find `{ |`, then scan identifier, then `|`, then `\s*<same_id>\.\w+\s*}`
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;
            while j < len {
                // Look for `{ |`
                if bytes[j] == b'{' {
                    let mut k = j + 1;
                    // Skip spaces
                    while k < len && bytes[k] == b' ' { k += 1; }
                    if k < len && bytes[k] == b'|' {
                        k += 1;
                        // Skip spaces
                        while k < len && bytes[k] == b' ' { k += 1; }
                        // Read block param name
                        let param_start = k;
                        while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                            k += 1;
                        }
                        let param_end = k;
                        if param_end == param_start {
                            j += 1;
                            continue;
                        }
                        let param = &line[param_start..param_end];
                        // Skip spaces then `|`
                        while k < len && bytes[k] == b' ' { k += 1; }
                        if k >= len || bytes[k] != b'|' {
                            j += 1;
                            continue;
                        }
                        k += 1;
                        // Skip spaces
                        while k < len && bytes[k] == b' ' { k += 1; }
                        // Match `param.method` (same param)
                        let body_start = k;
                        while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                            k += 1;
                        }
                        let body_param = &line[body_start..k];
                        if body_param != param {
                            j += 1;
                            continue;
                        }
                        // Check `.`
                        if k >= len || bytes[k] != b'.' {
                            j += 1;
                            continue;
                        }
                        k += 1;
                        // Read method name
                        let method_start = k;
                        while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_' || bytes[k] == b'?' || bytes[k] == b'!') {
                            k += 1;
                        }
                        let method_end = k;
                        if method_end == method_start {
                            j += 1;
                            continue;
                        }
                        // Skip spaces then `}`
                        while k < len && bytes[k] == b' ' { k += 1; }
                        if k < len && bytes[k] == b'}' {
                            // Violation found
                            let line_start = ctx.line_start_offsets[i];
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Use `(&:{})` instead of block `{{ |{}| {}.{} }}`.",
                                    &line[method_start..method_end],
                                    param, param,
                                    &line[method_start..method_end]
                                ),
                                range: TextRange::new(line_start + j as u32, line_start + k as u32 + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
                j += 1;
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct LambdaCall;

/// Returns true if the byte at `pos` in `bytes` is inside a string literal.
fn in_string_at(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos && i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => {
                return false;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

impl Rule for LambdaCall {
    fn name(&self) -> &'static str {
        "Style/LambdaCall"
    }

    /// Disabled by default: without AST type information we cannot reliably
    /// distinguish lambda objects from regular objects, making text-based
    /// detection too noisy. Users who care can enable it explicitly.
    fn default_enabled(&self) -> bool {
        false
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Search for `.call(` where the preceding char is a word character.
            // This distinguishes lambda.call(args) from obj.call_something(args).
            let pattern = b".call(";
            let mut search = 0usize;
            while search + pattern.len() <= bytes.len() {
                if bytes[search..].starts_with(pattern) {
                    // The char before `.call(` must be a word char (identifier end)
                    let preceding_ok = search > 0
                        && (bytes[search - 1].is_ascii_alphanumeric() || bytes[search - 1] == b'_');

                    // The char after `(` must NOT be part of a method name suffix,
                    // i.e. `.call(` should NOT be followed by something that makes
                    // `.call` a prefix of a longer method name. We check that the
                    // sequence is exactly `.call(` — the `(` already ensures this.

                    if preceding_ok && !in_string_at(bytes, search) {
                        let abs_start = (line_start + search) as u32;
                        let abs_end = (line_start + search + pattern.len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Prefer lambda.(args) over lambda.call(args).".into(),
                            range: TextRange::new(abs_start, abs_end),
                            severity: Severity::Warning,
                        });
                        search += pattern.len();
                        continue;
                    }
                }
                search += 1;
            }
        }

        diags
    }
}

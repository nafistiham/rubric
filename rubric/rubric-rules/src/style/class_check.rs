use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ClassCheck;

/// Returns the byte index of a real `#` comment character in `line`, skipping
/// `#` that appear inside string literals.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut interp_depth: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(b'"') if bytes[i..].starts_with(b"#{") => {
                interp_depth += 1;
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d && interp_depth == 0 => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        if interp_depth > 0 && bytes[i] == b'}' {
            interp_depth -= 1;
        }
        i += 1;
    }
    None
}

/// Returns `true` if `pos` in `bytes` is inside a string literal.
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
            None if bytes[i] == b'#' => return false,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

impl Rule for ClassCheck {
    fn name(&self) -> &'static str {
        "Style/ClassCheck"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let patterns: &[(&[u8], &str)] = &[
            (
                b".kind_of?",
                "Prefer Object#is_a? over Object#kind_of?.",
            ),
            (
                b".instance_of?",
                "Prefer Object#is_a? over Object#instance_of?.",
            ),
        ];

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];
            let bytes = scan_slice.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            for (pattern, message) in patterns {
                let mut search = 0usize;
                while search < bytes.len() {
                    if let Some(rel) = bytes[search..]
                        .windows(pattern.len())
                        .position(|w| w == *pattern)
                    {
                        let abs = search + rel;

                        // Ensure the char after the method name is not alphanumeric or `_`
                        // (to avoid matching e.g. `.kind_of?_extended`)
                        let after = abs + pattern.len();
                        let after_ok = after >= bytes.len()
                            || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_');

                        if after_ok && !in_string_at(bytes, abs) {
                            let start = (line_start + abs) as u32;
                            let end = start + pattern.len() as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: (*message).into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                        }

                        search = abs + pattern.len();
                    } else {
                        break;
                    }
                }
            }
        }

        diags
    }
}

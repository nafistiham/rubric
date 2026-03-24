use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CollectionMethods;

/// Returns the byte index of the comment character `#` in the line,
/// ignoring `#` inside string literals or interpolations.
/// Returns `None` if no real comment exists.
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

/// Returns true if byte position `pos` in `bytes` is inside a string literal.
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

/// Mapping of non-preferred method names to (preferred_name, message).
static ALIASES: &[(&[u8], &str, &str)] = &[
    (b"collect_concat", "flat_map", "Prefer flat_map over collect_concat."),
    (b"collect",        "map",      "Prefer map over collect."),
    (b"detect",         "find",     "Prefer find over detect."),
    (b"find_all",       "select",   "Prefer select over find_all."),
    (b"inject",         "reduce",   "Prefer reduce over inject."),
];

impl Rule for CollectionMethods {
    fn name(&self) -> &'static str {
        "Style/CollectionMethods"
    }

    /// Disabled by default in RuboCop — this cop enforces opinionated
    /// aliases (collect → map, detect → find, etc.) that many teams
    /// intentionally allow. Mirror RuboCop's default.
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

            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];
            let bytes = scan_slice.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Iterate aliases in order (collect_concat before collect to avoid partial match)
            for (method, _preferred, message) in ALIASES {
                // Pattern: `.` followed by the method name
                let dot_method_len = 1 + method.len(); // `.` + method
                let mut search = 0usize;

                while search + dot_method_len <= bytes.len() {
                    // Find `.method` sequence
                    let window = &bytes[search..];
                    let found = window
                        .windows(dot_method_len)
                        .position(|w| w[0] == b'.' && &w[1..] == *method);

                    let Some(rel) = found else { break };
                    let abs = search + rel;

                    // The char before `.` must be word char, `)`, or `]` — i.e., receiver
                    let prev_ok = if abs == 0 {
                        false
                    } else {
                        let p = bytes[abs - 1];
                        p.is_ascii_alphanumeric() || p == b'_' || p == b')' || p == b']'
                    };

                    // The char after the method name must be non-word (word boundary)
                    let after = abs + dot_method_len;
                    let after_ok = after >= bytes.len()
                        || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_');

                    if prev_ok && after_ok && !in_string_at(bytes, abs) {
                        let start = (line_start + abs) as u32;
                        let end = start + dot_method_len as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: (*message).into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                    }

                    search = abs + dot_method_len;
                }
            }
        }

        diags
    }
}

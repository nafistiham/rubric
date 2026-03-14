use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DateTime;

impl Rule for DateTime {
    fn name(&self) -> &'static str {
        "Style/DateTime"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Skip require statements
            if trimmed.starts_with("require ") || trimmed.starts_with("require_relative ") {
                continue;
            }

            // Skip class definition lines for DateTime itself
            if trimmed.starts_with("class DateTime") {
                continue;
            }

            // Find real comment boundary — scan only code portion
            let scan_end = comment_start(line).unwrap_or(line.len());
            let code_slice = &line[..scan_end];

            // Search for the word `DateTime` in the code portion
            let needle = b"DateTime";
            let bytes = code_slice.as_bytes();
            let mut search = 0usize;

            while search < bytes.len() {
                let found = bytes[search..]
                    .windows(needle.len())
                    .position(|w| w == needle);

                let Some(rel) = found else { break };
                let abs = search + rel;

                // Check word boundary before: must not be alphanumeric or `_`
                let before_ok = abs == 0
                    || (!bytes[abs - 1].is_ascii_alphanumeric() && bytes[abs - 1] != b'_');

                // Check word boundary after
                let after = abs + needle.len();
                let after_ok = after >= bytes.len()
                    || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_');

                if before_ok && after_ok && !in_string_at(bytes, abs) {
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let start = (line_start + abs) as u32;
                    let end = start + needle.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Prefer Time over DateTime. DateTime is considered legacy."
                            .into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                }

                search = abs + needle.len();
            }
        }

        diags
    }
}

/// Returns the byte index of the first real `#` comment character on the line,
/// ignoring `#` inside string literals.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(b'"') if bytes[i..].starts_with(b"#{") => {
                // interpolation — skip the `#{`; we do not track depth here
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
            None if bytes[i] == b'#' => return Some(i),
            None => {}
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
            None if bytes[i] == b'#' => return false,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

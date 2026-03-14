use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ColonMethodCall;

impl Rule for ColonMethodCall {
    fn name(&self) -> &'static str {
        "Style/ColonMethodCall"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Find real comment boundary — scan only code portion
            let scan_end = comment_start(line).unwrap_or(line.len());
            let code_slice = &line[..scan_end];
            let bytes = code_slice.as_bytes();

            // Search for `::` occurrences and check surrounding characters
            let needle = b"::";
            let mut search = 0usize;

            while search + 2 <= bytes.len() {
                let found = bytes[search..].windows(2).position(|w| w == needle);
                let Some(rel) = found else { break };
                let abs = search + rel;

                // The character before `::` must be part of an uppercase-starting identifier.
                // Walk backwards to find the start of the identifier before `::`.
                if abs == 0 {
                    search = abs + 2;
                    continue;
                }

                // Ensure the immediately preceding char is alphanumeric/`_` (part of identifier)
                if !bytes[abs - 1].is_ascii_alphanumeric() && bytes[abs - 1] != b'_' {
                    search = abs + 2;
                    continue;
                }

                // Walk back to find the start of the identifier on the left of `::`
                let mut id_start = abs;
                while id_start > 0
                    && (bytes[id_start - 1].is_ascii_alphanumeric() || bytes[id_start - 1] == b'_')
                {
                    id_start -= 1;
                }

                // The identifier on the left must start with an uppercase letter
                if !bytes[id_start].is_ascii_uppercase() {
                    search = abs + 2;
                    continue;
                }

                // The character after `::` must be a lowercase letter (method, not constant)
                let after = abs + 2;
                if after >= bytes.len() || !bytes[after].is_ascii_lowercase() {
                    search = abs + 2;
                    continue;
                }

                // Skip if the position is inside a string literal
                if in_string_at(bytes, abs) {
                    search = abs + 2;
                    continue;
                }

                let line_start = ctx.line_start_offsets[i] as usize;

                // Determine end of the right-hand identifier for the range
                let mut id_end = after;
                while id_end < bytes.len()
                    && (bytes[id_end].is_ascii_alphanumeric() || bytes[id_end] == b'_')
                {
                    id_end += 1;
                }

                let start = (line_start + abs) as u32;
                let end = (line_start + id_end) as u32;

                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Do not use :: for method calls; use . instead.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });

                search = id_end;
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

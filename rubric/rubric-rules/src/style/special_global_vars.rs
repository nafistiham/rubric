use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpecialGlobalVars;

/// Mapping from Perl-style special global variables to their English equivalents.
const GLOBAL_VAR_MAP: &[(&str, &str)] = &[
    ("$:", "$LOAD_PATH"),
    ("$\"", "$LOADED_FEATURES"),
    ("$0", "$PROGRAM_NAME"),
    ("$!", "$ERROR_INFO"),
    ("$@", "$ERROR_POSITION"),
    ("$;", "$FIELD_SEPARATOR"),
    ("$,", "$OUTPUT_FIELD_SEPARATOR"),
    // $\ is a backslash — represented as two chars in source
    ("$\\", "$OUTPUT_RECORD_SEPARATOR"),
    ("$.", "$INPUT_LINE_NUMBER"),
    ("$_", "$LAST_READ_LINE"),
    ("$&", "$MATCH"),
    ("$~", "$LAST_MATCH_INFO"),
    ("$>", "$DEFAULT_OUTPUT"),
];

impl Rule for SpecialGlobalVars {
    fn name(&self) -> &'static str {
        "Style/SpecialGlobalVars"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Determine where the real comment starts so we don't scan into it
            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];
            let line_start = ctx.line_start_offsets[i] as usize;

            for (perl_name, english_name) in GLOBAL_VAR_MAP {
                let needle = perl_name.as_bytes();
                let bytes = scan_slice.as_bytes();
                let mut search = 0usize;

                while search < bytes.len() {
                    if let Some(rel) = bytes[search..]
                        .windows(needle.len())
                        .position(|w| w == needle)
                    {
                        let abs = search + rel;

                        // Make sure this is not inside a string or percent literal
                        if !in_string_at(bytes, abs) && !in_percent_literal_at(bytes, abs) {
                            // Ensure the character after the global is not alphanumeric or `_`
                            // (to avoid matching `$0` inside `$0abc` incorrectly for single-char vars)
                            let after = abs + needle.len();
                            let boundary_ok = after >= bytes.len()
                                || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_');

                            if boundary_ok {
                                let start = (line_start + abs) as u32;
                                let end = start + needle.len() as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!(
                                        "Prefer {} over {}.",
                                        english_name, perl_name
                                    ),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                            }
                        }

                        search = abs + needle.len();
                    } else {
                        break;
                    }
                }
            }
        }

        diags
    }
}

/// Returns true if the byte position `pos` in `bytes` is inside a string literal.
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
                // Real comment hit before pos — nothing after is code
                return false;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Returns true if position `pos` in `bytes` is inside a `%r`, `%w`, etc.
/// percent literal (e.g. `%r!pattern!`, `%w(foo bar)`).
fn in_percent_literal_at(bytes: &[u8], pos: usize) -> bool {
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'%' {
            let kind = bytes[i + 1];
            if matches!(kind, b'r' | b'w' | b'W' | b'i' | b'I' | b'q' | b'Q' | b'x' | b's') {
                let open = bytes[i + 2];
                let close = match open {
                    b'(' => b')',
                    b'[' => b']',
                    b'{' => b'}',
                    b'<' => b'>',
                    _ => open,
                };
                let lit_start = i + 3;
                // Find the matching close delimiter
                let mut j = lit_start;
                while j < bytes.len() {
                    if bytes[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if bytes[j] == close {
                        let lit_end = j;
                        if pos > i && pos <= lit_end {
                            return true;
                        }
                        i = j + 1;
                        break;
                    }
                    j += 1;
                }
                if j >= bytes.len() {
                    // Unclosed literal — treat remainder as inside literal
                    if pos > i {
                        return true;
                    }
                    return false;
                }
                continue;
            }
        }
        i += 1;
    }
    false
}

/// Returns the index of the comment character `#` on the line, ignoring
/// `#` that appear inside string literals.
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

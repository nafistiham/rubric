use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SymbolLiteral;

/// Returns true if the byte at `pos` in `bytes` is inside a string literal
/// (handles single and double quoted strings, ignoring escaped quotes).
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
                // Real comment — nothing past here is code
                return false;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Returns true if all bytes in `name` are word characters (ASCII alphanumeric or `_`).
fn is_simple_symbol_name(name: &[u8]) -> bool {
    !name.is_empty() && name.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_')
}

impl Rule for SymbolLiteral {
    fn name(&self) -> &'static str {
        "Style/SymbolLiteral"
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

            // Look for `:"` or `:'` patterns
            let mut search = 0usize;
            while search + 2 < bytes.len() {
                if bytes[search] == b':' {
                    let quote = bytes[search + 1];
                    if quote == b'"' || quote == b'\'' {
                        // Skip if the `:` is inside a string literal
                        if !in_string_at(bytes, search) {
                            // Collect the symbol name bytes between the quotes
                            let name_start = search + 2;
                            let mut end = name_start;
                            while end < bytes.len() && bytes[end] != quote {
                                if bytes[end] == b'\\' {
                                    end += 1; // skip escaped char
                                }
                                end += 1;
                            }
                            // `end` now points at the closing quote (or past EOF)
                            if end < bytes.len() && bytes[end] == quote {
                                let name = &bytes[name_start..end];
                                if is_simple_symbol_name(name) {
                                    let abs_start = (line_start + search) as u32;
                                    // Range covers :"name" or :'name'
                                    let abs_end = (line_start + end + 1) as u32;
                                    diags.push(Diagnostic {
                                        rule: self.name(),
                                        message: "Use the simpler symbol literal instead of the quoted version.".into(),
                                        range: TextRange::new(abs_start, abs_end),
                                        severity: Severity::Warning,
                                    });
                                    search = end + 1;
                                    continue;
                                }
                            }
                        }
                        search += 1;
                        continue;
                    }
                }
                search += 1;
            }
        }

        diags
    }
}

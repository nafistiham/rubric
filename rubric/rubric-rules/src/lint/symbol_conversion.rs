use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SymbolConversion;

/// Returns true if `bytes[pos..]` starts with a word character (alphanumeric or `_`).
fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

impl Rule for SymbolConversion {
    fn name(&self) -> &'static str {
        "Lint/SymbolConversion"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[line_idx] as usize;

            let mut i = 0;
            while i < bytes.len() {
                // Skip comments
                if bytes[i] == b'#' {
                    break;
                }

                // Skip string literals
                if bytes[i] == b'"' || bytes[i] == b'\'' {
                    let quote = bytes[i];
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == b'\\' {
                            i += 2;
                            continue;
                        }
                        if bytes[i] == quote {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                    // After the closing quote, check if `.to_sym` follows
                    if bytes[i..].starts_with(b".to_sym") {
                        let after = i + b".to_sym".len();
                        let after_ok = after >= bytes.len() || !is_word_char(bytes[after]);
                        if after_ok {
                            let start = (line_start + i) as u32;
                            let end = start + b".to_sym".len() as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Unnecessary use of to_sym.".into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    continue;
                }

                // Detect symbol literal: `:word`
                if bytes[i] == b':' && i + 1 < bytes.len() && is_word_char(bytes[i + 1]) {
                    let sym_start = i;
                    i += 1;
                    // Skip the symbol name characters
                    while i < bytes.len() && is_word_char(bytes[i]) {
                        i += 1;
                    }
                    // Check if `.to_sym` follows
                    if bytes[i..].starts_with(b".to_sym") {
                        let after = i + b".to_sym".len();
                        let after_ok = after >= bytes.len() || !is_word_char(bytes[after]);
                        if after_ok {
                            let start = (line_start + sym_start) as u32;
                            let end = (line_start + i + b".to_sym".len()) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Unnecessary use of to_sym.".into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    continue;
                }

                i += 1;
            }
        }

        diags
    }
}

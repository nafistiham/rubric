use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SymbolArray;

impl Rule for SymbolArray {
    fn name(&self) -> &'static str {
        "Style/SymbolArray"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip comment lines — symbol arrays in doc examples must not be flagged
            if line.trim_start().starts_with('#') {
                continue;
            }
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;
            let mut in_string: Option<u8> = None;

            while j < len {
                // ── String state: skip characters inside string literals ──
                match in_string {
                    Some(_) if bytes[j] == b'\\' => { j += 2; continue; }
                    Some(delim) if bytes[j] == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if bytes[j] == b'"' || bytes[j] == b'\'' || bytes[j] == b'`' => {
                        in_string = Some(bytes[j]); j += 1; continue;
                    }
                    None if bytes[j] == b'#' => break, // inline comment — stop
                    None => {}
                }

                // Look for `[` to start an array literal
                if bytes[j] != b'[' {
                    j += 1;
                    continue;
                }

                let array_start = j;
                j += 1;

                // Skip whitespace
                while j < len && bytes[j] == b' ' { j += 1; }

                // Check if first element is a symbol `:word`
                if j >= len || bytes[j] != b':' {
                    j = array_start + 1;
                    continue;
                }

                // Try to parse the full array as all symbols
                let mut k = j;
                let mut symbol_count = 0;
                let mut valid = true;

                while k < len {
                    // Skip whitespace
                    while k < len && bytes[k] == b' ' { k += 1; }

                    if k >= len { valid = false; break; }

                    if bytes[k] == b']' {
                        // End of array
                        break;
                    }

                    // Expect `:symbol`
                    if bytes[k] != b':' {
                        valid = false;
                        break;
                    }
                    k += 1;

                    // Read symbol name
                    let sym_start = k;
                    while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                        k += 1;
                    }

                    if k == sym_start {
                        // Empty symbol name
                        valid = false;
                        break;
                    }

                    symbol_count += 1;

                    // Skip whitespace
                    while k < len && bytes[k] == b' ' { k += 1; }

                    // Expect `,` or `]`
                    if k >= len { valid = false; break; }
                    if bytes[k] == b']' {
                        break;
                    } else if bytes[k] == b',' {
                        k += 1;
                    } else {
                        valid = false;
                        break;
                    }
                }

                if valid && symbol_count >= 2 && k < len && bytes[k] == b']' {
                    let start = (line_start + array_start) as u32;
                    let end = (line_start + k + 1) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `%i[]` for arrays of symbols.".into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                    j = k + 1;
                } else {
                    j = array_start + 1;
                }
            }
        }

        diags
    }
}

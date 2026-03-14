use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct VariableName;

/// Returns true if the name is in camelCase:
/// - starts with a lowercase letter
/// - contains at least one uppercase letter after the first character
fn is_camel_case(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes[1..].iter().any(|&b| b.is_ascii_uppercase())
}


impl Rule for VariableName {
    fn name(&self) -> &'static str {
        "Naming/VariableName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                // String tracking
                if let Some(delim) = in_string {
                    match b {
                        b'\\' => {
                            j += 2;
                            continue;
                        }
                        c if c == delim => {
                            in_string = None;
                        }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                match b {
                    b'"' | b'\'' | b'`' => {
                        in_string = Some(b);
                        j += 1;
                        continue;
                    }
                    b'#' => break, // inline comment
                    // Skip instance/class/global variables
                    b'@' | b'$' => {
                        // Skip over the sigil and the identifier
                        j += 1;
                        while j < n && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'@') {
                            j += 1;
                        }
                        continue;
                    }
                    _ => {}
                }

                // Look for an identifier starting with a lowercase letter
                if b.is_ascii_lowercase() || b == b'_' {
                    let id_start = j;
                    while j < n && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                        j += 1;
                    }
                    let id_end = j;
                    let name = std::str::from_utf8(&bytes[id_start..id_end]).unwrap_or("");

                    // Skip whitespace after the identifier
                    let mut k = j;
                    while k < n && (bytes[k] == b' ' || bytes[k] == b'\t') {
                        k += 1;
                    }

                    // Check for `=` that is not part of `==`, `!=`, `<=`, `>=`, `=>`, `+=`, `-=`, etc.
                    if k < n && bytes[k] == b'=' {
                        // Must not be followed by `=` or `>`
                        let next = if k + 1 < n { bytes[k + 1] } else { 0 };
                        if next != b'=' && next != b'>' {
                            // Also make sure the char before `=` is not `!`, `<`, `>`, `+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `~`
                            // (compound assignment operators): these would be things like `+=`, `-=`, etc.
                            // Since we already extracted the identifier and skipped whitespace,
                            // the `=` is a plain assignment if we get here.
                            if is_camel_case(name) {
                                let start = (line_start + id_start) as u32;
                                let end = (line_start + id_end) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use snake_case for variable names.".to_string(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                    continue;
                }

                // Skip over identifiers starting with uppercase (constants, method calls)
                if b.is_ascii_uppercase() {
                    while j < n && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                        j += 1;
                    }
                    continue;
                }

                j += 1;
            }
        }

        diags
    }
}

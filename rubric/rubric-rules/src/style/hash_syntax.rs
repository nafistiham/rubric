use rubric_core::{Diagnostic, Fix, LintContext, Rule, Severity, TextRange};

pub struct HashSyntax;

impl Rule for HashSyntax {
    fn name(&self) -> &'static str {
        "Style/HashSyntax"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < len {
                let b = bytes[j];

                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break,
                    None => {}
                }

                // Look for `:symbol =>` pattern: starts with `:` followed by word chars then ` =>`
                if b == b':' {
                    // Read the symbol name (word chars)
                    let mut k = j + 1;
                    while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                        k += 1;
                    }

                    // Check if symbol name is non-empty and followed by ` =>`
                    if k > j + 1 {
                        // Skip spaces
                        let mut m = k;
                        while m < len && bytes[m] == b' ' {
                            m += 1;
                        }
                        if m + 1 < len && bytes[m] == b'=' && bytes[m+1] == b'>' {
                            let pos = (line_start + j) as u32;
                            let end = (line_start + m + 2) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Use the new hash syntax (`key: value`) instead of `{:key => value}`.".into(),
                                range: TextRange::new(pos, end),
                                severity: Severity::Warning,
                            });
                            j = m + 2;
                            continue;
                        }
                    }
                }

                j += 1;
            }
        }

        diags
    }

    fn fix(&self, _diag: &Diagnostic) -> Option<Fix> {
        None // TODO: implement :sym => val -> sym: val transformation
    }
}

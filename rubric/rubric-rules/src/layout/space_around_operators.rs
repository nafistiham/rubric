use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceAroundOperators;

impl Rule for SpaceAroundOperators {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundOperators"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;
            let mut in_string: Option<u8> = None;
            let mut in_comment = false;

            while j < len {
                let b = bytes[j];

                // Handle string state
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => { in_comment = true; break; }
                    None => {}
                }

                if in_comment { break; }

                // Check for multi-char operators first
                if j + 1 < len {
                    let two = &bytes[j..j+2];
                    // Skip compound operators: ==, !=, <=, >=, &&, ||, +=, -=, *=, /=, **
                    if two == b"==" || two == b"!=" || two == b"<=" || two == b">="
                        || two == b"&&" || two == b"||" || two == b"+=" || two == b"-="
                        || two == b"*=" || two == b"/=" || two == b"**"
                    {
                        // Check spacing around these two-char operators
                        let prev_ok = j == 0 || bytes[j-1] == b' ' || bytes[j-1] == b'\t';
                        let next_ok = j + 2 >= len || bytes[j+2] == b' ' || bytes[j+2] == b'\t' || bytes[j+2] == b'\n';
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Operator `{}` should be surrounded by spaces.",
                                    std::str::from_utf8(two).unwrap_or("?")
                                ),
                                range: TextRange::new(pos, pos + 2),
                                severity: Severity::Warning,
                            });
                        }
                        j += 2;
                        continue;
                    }

                    // Skip ->
                    if two == b"->" {
                        j += 2;
                        continue;
                    }
                }

                // Single-char operators: = + - * / < >
                // Skip if part of a symbol literal `:foo`, string delimiter, etc.
                match b {
                    b'=' => {
                        // Already handled ==, !=, <=, >= above
                        // Check this = is not part of those (we'd have advanced past them)
                        // Check previous char to skip !=, <=, >= = at position j-1
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        if prev == b'!' || prev == b'<' || prev == b'>' || prev == b'=' {
                            j += 1;
                            continue;
                        }
                        // Check for => (hash rocket)
                        let next = if j + 1 < len { bytes[j+1] } else { 0 };
                        if next == b'>' {
                            j += 1;
                            continue;
                        }
                        // Check spacing
                        let prev_ok = j == 0 || prev == b' ' || prev == b'\t';
                        let next_ok = next == b' ' || next == b'\t' || next == 0;
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Operator `=` should be surrounded by spaces.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    b'+' | b'-' | b'*' | b'/' => {
                        // Skip if next is = (+=, -=, *=, /=) — handled above
                        if j + 1 < len && bytes[j+1] == b'=' {
                            j += 1;
                            continue;
                        }
                        // Skip ** — handled above
                        if b == b'*' && j + 1 < len && bytes[j+1] == b'*' {
                            j += 1;
                            continue;
                        }
                        // Skip unary: operator at start of line or after (, [, ,, space, = operator
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        if prev == b'(' || prev == b'[' || prev == b',' || prev == b' '
                            || prev == b'\t' || prev == 0 || prev == b'=' || prev == b'+'
                            || prev == b'-' || prev == b'*' || prev == b'/'
                        {
                            j += 1;
                            continue;
                        }
                        // Skip if - is part of a negative number literal
                        let next = if j + 1 < len { bytes[j+1] } else { 0 };
                        if (b == b'-' || b == b'+') && next.is_ascii_digit() && prev != b' ' {
                            j += 1;
                            continue;
                        }
                        let prev_ok = prev == b' ' || prev == b'\t';
                        let next_ok = next == b' ' || next == b'\t';
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!("Operator `{}` should be surrounded by spaces.", b as char),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    _ => {}
                }

                j += 1;
            }
        }

        diags
    }
}

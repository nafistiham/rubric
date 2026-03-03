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
            let mut in_regex = false;

            while j < len {
                let b = bytes[j];

                // ── Skip %r{...} percent-regex literals ──────────────────────
                if j + 1 < len && b == b'%' && bytes[j+1] == b'r' {
                    j += 2;
                    if j < len {
                        let delim = bytes[j];
                        j += 1;
                        if delim == b'{' {
                            let mut depth = 1usize;
                            while j < len && depth > 0 {
                                match bytes[j] {
                                    b'\\' => { j += 2; }
                                    b'{' => { depth += 1; j += 1; }
                                    b'}' => { depth -= 1; j += 1; }
                                    _ => { j += 1; }
                                }
                            }
                        } else {
                            while j < len && bytes[j] != delim {
                                if bytes[j] == b'\\' { j += 2; } else { j += 1; }
                            }
                            if j < len { j += 1; }
                        }
                    }
                    continue;
                }

                // ── Regex state: skip until unescaped `/` ────────────────────
                if in_regex {
                    match b {
                        b'\\' => { j += 2; continue; }
                        b'/' => { in_regex = false; }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                // ── String state ─────────────────────────────────────────────
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment — stop scanning
                    None => {}
                }

                // ── 3-char compound operators: ||=, &&= ──────────────────────
                if j + 2 < len {
                    let three = &bytes[j..j+3];
                    if three == b"||=" || three == b"&&=" {
                        let prev_ok = j == 0 || bytes[j-1] == b' ' || bytes[j-1] == b'\t';
                        let next_ok = j + 3 >= len || bytes[j+3] == b' ' || bytes[j+3] == b'\t';
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Operator `{}` should be surrounded by spaces.",
                                    std::str::from_utf8(three).unwrap_or("?")
                                ),
                                range: TextRange::new(pos, pos + 3),
                                severity: Severity::Warning,
                            });
                        }
                        j += 3;
                        continue;
                    }
                }

                // ── 2-char operators ─────────────────────────────────────────
                if j + 1 < len {
                    let two = &bytes[j..j+2];

                    // ** — exponentiation OR double-splat.
                    // It's a double-splat (**opts / **hash) when:
                    //   - the next char is an identifier char, AND
                    //   - the preceding char is NOT an identifier/digit/closing bracket
                    //     (i.e. we're after `(`, `,`, space, `{`, etc.)
                    if two == b"**" {
                        let next_b = if j + 2 < len { bytes[j+2] } else { 0 };
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        if (next_b.is_ascii_alphabetic() || next_b == b'_')
                            && !prev.is_ascii_alphanumeric() && prev != b')' && prev != b']'
                        {
                            // double-splat: **opts / **hash — no spacing required
                            j += 2;
                            continue;
                        }
                        // exponentiation — no spacing required (RuboCop default: no_space style)
                        j += 2;
                        continue;
                    }

                    // Skip ->
                    if two == b"->" {
                        j += 2;
                        continue;
                    }

                    // Remaining two-char operators: ==, !=, <=, >=, &&, ||, +=, -=, *=, /=
                    if two == b"==" || two == b"!=" || two == b"<=" || two == b">="
                        || two == b"&&" || two == b"||" || two == b"+=" || two == b"-="
                        || two == b"*=" || two == b"/="
                    {
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
                }

                // ── Single-char operators: = + - * / < > ────────────────────
                match b {
                    b'=' => {
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        // Skip trailing char of !=, <=, >=, ==
                        if prev == b'!' || prev == b'<' || prev == b'>' || prev == b'=' {
                            j += 1;
                            continue;
                        }
                        // Skip => (hash rocket)
                        let next = if j + 1 < len { bytes[j+1] } else { 0 };
                        if next == b'>' {
                            j += 1;
                            continue;
                        }
                        // Skip =~ (regex match operator)
                        if next == b'~' {
                            j += 1;
                            continue;
                        }
                        // Skip setter method definitions: def foo=(val)
                        // The `=` is part of the method name when followed by `(`
                        if next == b'(' {
                            j += 1;
                            continue;
                        }
                        // Skip setter method calls: self.foo=val, obj.attr=val
                        // Scan backward through identifier chars; if preceded by `.` it's a setter call
                        {
                            let mut k = j;
                            while k > 0 && (bytes[k-1].is_ascii_alphanumeric() || bytes[k-1] == b'_') {
                                k -= 1;
                            }
                            if k > 0 && bytes[k-1] == b'.' {
                                j += 1;
                                continue;
                            }
                        }
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
                    b'/' => {
                        // `/` after an operator/open-paren/space is a regex start.
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        if prev == b'=' || prev == b'(' || prev == b',' || prev == b'['
                            || prev == b' ' || prev == b'\t' || prev == 0
                        {
                            in_regex = true;
                            j += 1;
                            continue;
                        }
                        // Otherwise treat as division and check spacing.
                        let next = if j + 1 < len { bytes[j+1] } else { 0 };
                        // Skip if it's /= (already handled in two-char section, but defensive)
                        if next == b'=' {
                            j += 1;
                            continue;
                        }
                        let prev_ok = prev == b' ' || prev == b'\t';
                        let next_ok = next == b' ' || next == b'\t';
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Operator `/` should be surrounded by spaces.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    b'+' | b'-' | b'*' => {
                        // Skip if next is = (+=, -=, *=) — handled in two-char section
                        if j + 1 < len && bytes[j+1] == b'=' {
                            j += 1;
                            continue;
                        }
                        // Skip * when next is * — already handled as ** above
                        if b == b'*' && j + 1 < len && bytes[j+1] == b'*' {
                            j += 1;
                            continue;
                        }
                        // Skip unary: operator at start of line or after (, [, ,, space, =, operator
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        if prev == b'(' || prev == b'[' || prev == b',' || prev == b' '
                            || prev == b'\t' || prev == 0 || prev == b'=' || prev == b'+'
                            || prev == b'-' || prev == b'*' || prev == b'/'
                        {
                            j += 1;
                            continue;
                        }
                        // Skip negative/positive number literals
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

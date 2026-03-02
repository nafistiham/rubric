use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UnusedMethodArgument;

impl Rule for UnusedMethodArgument {
    fn name(&self) -> &'static str {
        "Lint/UnusedMethodArgument"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();

            // Look for `def method_name(args)` pattern
            if !trimmed.starts_with("def ") {
                i += 1;
                continue;
            }

            // Parse the argument list
            let def_line = trimmed;
            let paren_open = match def_line.find('(') {
                Some(p) => p,
                None => { i += 1; continue; }
            };
            let paren_close = match def_line.rfind(')') {
                Some(p) => p,
                None => { i += 1; continue; }
            };

            if paren_close <= paren_open {
                i += 1;
                continue;
            }

            let args_str = &def_line[paren_open+1..paren_close];
            let args: Vec<&str> = args_str.split(',')
                .map(|a| a.trim())
                .filter(|a| !a.is_empty())
                .collect();

            // Find end of this def block
            let def_start = i;
            let mut depth = 1i32;
            let mut j = i + 1;
            while j < n && depth > 0 {
                let t = lines[j].trim();
                if t.starts_with("def ") || t.starts_with("if ") || t.starts_with("unless ")
                    || t.starts_with("do ") || t == "do" || t.starts_with("begin")
                    || t.starts_with("case ")
                    || t.starts_with("while ") || t == "while"
                    || t.starts_with("until ") || t == "until"
                    || t.starts_with("for ")
                    || t == "loop" || t.ends_with(" loop")
                {
                    depth += 1;
                }
                if t == "end" {
                    depth -= 1;
                }
                j += 1;
            }
            let def_end = j;

            // For each arg that doesn't start with `_`, check if it's used in the body
            for arg in &args {
                // Clean up arg (remove default values, splats, etc.)
                let arg_clean = arg.trim_start_matches('*').trim_start_matches('&');
                let arg_name_raw = arg_clean.split('=').next().unwrap_or("").trim();
                // Handle keyword args: "bar: 'default'" -> "bar", "bar:" -> "bar"
                let arg_name = if let Some(colon_pos) = arg_name_raw.find(':') {
                    arg_name_raw[..colon_pos].trim()
                } else {
                    arg_name_raw
                };

                if arg_name.is_empty() || arg_name.starts_with('_') {
                    continue;
                }

                // Check if arg_name appears in the method body
                let mut used = false;
                'body: for k in (def_start + 1)..def_end.min(n) {
                    let body_line = &lines[k];
                    let body_bytes = body_line.as_bytes();
                    let body_len = body_bytes.len();
                    let arg_bytes = arg_name.as_bytes();
                    let arg_len = arg_bytes.len();

                    let mut pos = 0;
                    let mut in_single_string = false; // Only skip single-quoted (no interpolation)

                    while pos < body_len {
                        let b = body_bytes[pos];

                        if in_single_string {
                            if b == b'\\' { pos += 2; continue; }
                            if b == b'\'' { in_single_string = false; }
                            pos += 1;
                            continue;
                        }

                        if b == b'\'' { in_single_string = true; pos += 1; continue; }

                        if b == b'#' {
                            // `#{` is string interpolation — not a comment, keep scanning
                            let next = if pos + 1 < body_len { body_bytes[pos + 1] } else { 0 };
                            if next != b'{' { break; } // actual comment — stop line
                        }

                        if pos + arg_len <= body_len && &body_bytes[pos..pos+arg_len] == arg_bytes {
                            let before_ok = pos == 0 || !body_bytes[pos-1].is_ascii_alphanumeric() && body_bytes[pos-1] != b'_';
                            let after_ok = pos + arg_len >= body_len
                                || !body_bytes[pos+arg_len].is_ascii_alphanumeric() && body_bytes[pos+arg_len] != b'_';
                            if before_ok && after_ok {
                                used = true;
                                break 'body;
                            }
                        }
                        pos += 1;
                    }
                }

                if !used {
                    // Flag the argument on the def line
                    let line_start = ctx.line_start_offsets[def_start] as usize;
                    let indent = line.len() - trimmed.len();
                    // Find the arg position in the original line
                    let def_offset = indent + paren_open + 1;
                    // Locate arg_name within the args_str
                    let arg_offset_in_args = args_str.find(arg_name).unwrap_or(0);
                    let pos = (line_start + def_offset + arg_offset_in_args) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Unused method argument `{}`.", arg_name),
                        range: TextRange::new(pos, pos + arg_name.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }

            i = def_end;
        }

        diags
    }
}

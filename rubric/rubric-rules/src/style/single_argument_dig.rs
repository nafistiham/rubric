use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SingleArgumentDig;

/// Returns true if the byte at `pos` in `bytes` is inside a string literal.
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
                return false;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Counts commas at the top nesting level within `bytes[start..end]`.
/// Nested parens/brackets/braces are skipped.
fn count_top_level_commas(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut depth = 0usize;
    let mut count = 0usize;
    let mut in_str: Option<u8> = None;
    let mut i = start;
    while i < end && i < bytes.len() {
        let b = bytes[i];
        match in_str {
            Some(_) if b == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if b == d => {
                in_str = None;
            }
            Some(_) => {}
            None if b == b'"' || b == b'\'' => {
                in_str = Some(b);
            }
            None if b == b'(' || b == b'[' || b == b'{' => {
                depth += 1;
            }
            None if b == b')' || b == b']' || b == b'}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            None if b == b',' && depth == 0 => {
                count += 1;
            }
            None => {}
        }
        i += 1;
    }
    count
}

impl Rule for SingleArgumentDig {
    fn name(&self) -> &'static str {
        "Style/SingleArgumentDig"
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

            // Search for `.dig(` pattern
            let pattern = b".dig(";
            let mut search = 0usize;
            while search + pattern.len() <= bytes.len() {
                if bytes[search..].starts_with(pattern) {
                    if !in_string_at(bytes, search) {
                        // Find the matching closing paren
                        let args_start = search + pattern.len();
                        let mut depth = 1usize;
                        let mut j = args_start;
                        let mut in_str_inner: Option<u8> = None;
                        while j < bytes.len() && depth > 0 {
                            let b = bytes[j];
                            match in_str_inner {
                                Some(_) if b == b'\\' => {
                                    j += 2;
                                    continue;
                                }
                                Some(d) if b == d => {
                                    in_str_inner = None;
                                }
                                Some(_) => {}
                                None if b == b'"' || b == b'\'' => {
                                    in_str_inner = Some(b);
                                }
                                None if b == b'(' => {
                                    depth += 1;
                                }
                                None if b == b')' => {
                                    depth -= 1;
                                }
                                None => {}
                            }
                            j += 1;
                        }
                        // `j` is one past the closing paren
                        let args_end = j - 1; // index of the closing paren

                        // Count commas — 0 commas means single argument.
                        // But splat `*expr` can expand to many args: skip those.
                        let comma_count = count_top_level_commas(bytes, args_start, args_end);
                        if comma_count == 0 {
                            // Make sure the argument list is non-empty
                            let arg_content = &bytes[args_start..args_end];
                            let first_non_ws = arg_content.iter().find(|&&b| !b.is_ascii_whitespace()).copied();
                            if !arg_content.iter().all(|b| b.is_ascii_whitespace())
                                && first_non_ws != Some(b'*')  // skip splat args
                            {
                                let abs_start = (line_start + search) as u32;
                                let abs_end = (line_start + j) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use dig only with 2+ arguments; use [] for single-key access.".into(),
                                    range: TextRange::new(abs_start, abs_end),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                        search = j;
                        continue;
                    }
                }
                search += 1;
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceInsideBlockBraces;

impl Rule for SpaceInsideBlockBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideBlockBraces"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let n = bytes.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            let mut pos = 0;
            let mut in_string: Option<u8> = None;
            // Stack recording whether each open `{` is a block (true) or hash (false).
            // Used to give the `}` check the same context as the `{` check.
            let mut brace_kind_stack: Vec<bool> = Vec::new();

            while pos < n {
                let b = bytes[pos];
                match in_string {
                    Some(_) if b == b'\\' => { pos += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; pos += 1; continue; }
                    Some(_) => { pos += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); pos += 1; continue; }
                    None if b == b'#' => break,
                    None => {}
                }

                // Skip percent literals that use `{` as delimiter.
                // Handles: %r{}, %{}, %w{}, %i{}, %W{}, %I{}, %q{}, %Q{}
                if b == b'%' && pos + 1 < n {
                    let next_b = bytes[pos + 1];
                    // Check for %r, %w, %i, %W, %I, %q, %Q followed by `{`,
                    // or bare `%` followed directly by `{`.
                    let (skip_tag_len, is_percent_brace) = match next_b {
                        b'r' | b'w' | b'i' | b'W' | b'I' | b'q' | b'Q' => {
                            // %X{ form
                            if pos + 2 < n && bytes[pos + 2] == b'{' {
                                (2usize, true)
                            } else if pos + 2 < n {
                                // %X with non-brace delimiter — skip as generic literal
                                let delim = bytes[pos + 2];
                                pos += 3;
                                while pos < n && bytes[pos] != delim {
                                    if bytes[pos] == b'\\' { pos += 2; } else { pos += 1; }
                                }
                                if pos < n { pos += 1; }
                                continue;
                            } else {
                                (0, false)
                            }
                        }
                        b'{' => {
                            // bare %{ form
                            (1usize, true)
                        }
                        _ => (0, false),
                    };

                    if is_percent_brace {
                        pos += skip_tag_len + 1; // skip `%`, optional letter(s), and `{`
                        let mut depth = 1usize;
                        while pos < n && depth > 0 {
                            match bytes[pos] {
                                b'\\' => { pos += 2; }
                                b'{' => { depth += 1; pos += 1; }
                                b'}' => { depth -= 1; pos += 1; }
                                _ => { pos += 1; }
                            }
                        }
                        continue;
                    }
                }

                // Skip /regex/ literals
                if b == b'/' {
                    let prev = if pos > 0 { bytes[pos - 1] } else { 0 };
                    if prev == b'=' || prev == b'(' || prev == b','
                        || prev == b'[' || prev == b' ' || prev == b'\t' || prev == 0
                    {
                        pos += 1;
                        while pos < n {
                            match bytes[pos] {
                                b'\\' => { pos += 2; }
                                b'/' => { pos += 1; break; }
                                _ => { pos += 1; }
                            }
                        }
                        continue;
                    }
                }

                if b == b'{' {
                    let next = if pos + 1 < n { bytes[pos + 1] } else { 0 };

                    // Determine if `{` opens a hash literal or a block.
                    // Hash contexts: `{` follows =, ,, (, [, {, :, or is the first
                    // non-whitespace character on the line.
                    let prev_nonspace = {
                        let mut p = pos;
                        let mut found = 0u8;
                        while p > 0 {
                            p -= 1;
                            if bytes[p] != b' ' && bytes[p] != b'\t' {
                                found = bytes[p];
                                break;
                            }
                        }
                        found
                    };

                    // Check if the preceding token is the keyword `in` (pattern matching).
                    // Scan backwards past whitespace; if the preceding word is `in`, treat as hash.
                    let preceded_by_in_keyword = {
                        let mut p = pos;
                        // skip whitespace before `{`
                        while p > 0 && (bytes[p - 1] == b' ' || bytes[p - 1] == b'\t') {
                            p -= 1;
                        }
                        // check if the two characters before the whitespace are `in`
                        // and that they are not part of a longer word
                        if p >= 2 && bytes[p - 2] == b'i' && bytes[p - 1] == b'n' {
                            // ensure `in` is a standalone word (not e.g. `begin`)
                            p - 2 == 0 || bytes[p - 3] == b' ' || bytes[p - 3] == b'\t'
                        } else {
                            false
                        }
                    };

                    let is_hash = preceded_by_in_keyword
                        || matches!(
                            prev_nonspace,
                            b'=' | b',' | b'(' | b'[' | b'{' | b':' | 0
                        )
                        || pos == line.len() - line.trim_start().len();

                    brace_kind_stack.push(!is_hash); // true = block

                    if !is_hash && next != b' ' && next != b'\n' && next != b'}' && next != 0 {
                        let flag_pos = (line_start + pos) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space missing inside block braces after `{`.".into(),
                            range: TextRange::new(flag_pos, flag_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }

                if b == b'}' {
                    // Only fire if this `}` closes a block brace (not a hash).
                    let is_block = brace_kind_stack.pop().unwrap_or(false);
                    if is_block && pos > 0 {
                        let prev = bytes[pos - 1];
                        if prev != b' ' && prev != b'\n' && prev != b'{' {
                            let flag_pos = (line_start + pos) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space missing inside block braces before `}`.".into(),
                                range: TextRange::new(flag_pos, flag_pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }

                pos += 1;
            }
        }

        diags
    }
}

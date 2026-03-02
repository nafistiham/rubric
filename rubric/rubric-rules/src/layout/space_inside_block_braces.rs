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

                // Skip %r{...} and other %r delimiters — regex literals
                if b == b'%' && pos + 1 < n && bytes[pos + 1] == b'r' {
                    pos += 2;
                    if pos < n {
                        let delim = bytes[pos];
                        pos += 1;
                        if delim == b'{' {
                            let mut depth = 1usize;
                            while pos < n && depth > 0 {
                                match bytes[pos] {
                                    b'\\' => { pos += 2; }
                                    b'{' => { depth += 1; pos += 1; }
                                    b'}' => { depth -= 1; pos += 1; }
                                    _ => { pos += 1; }
                                }
                            }
                        } else {
                            while pos < n && bytes[pos] != delim {
                                if bytes[pos] == b'\\' { pos += 2; } else { pos += 1; }
                            }
                            if pos < n { pos += 1; }
                        }
                    }
                    continue;
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
                    let is_hash = matches!(
                        prev_nonspace,
                        b'=' | b',' | b'(' | b'[' | b'{' | b':' | 0
                    ) || pos == line.len() - line.trim_start().len();

                    brace_kind_stack.push(!is_hash); // true = block

                    if !is_hash && next != b' ' && next != b'\n' && next != b'}' {
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

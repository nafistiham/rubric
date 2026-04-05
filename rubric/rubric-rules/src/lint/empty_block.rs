use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyBlock;

impl Rule for EmptyBlock {
    fn name(&self) -> &'static str {
        "Lint/EmptyBlock"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `{ }` — empty block with just whitespace
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;

            // Ruby keywords that can appear immediately before `{` but mean the `{}`
            // is an empty hash literal (return value, default, etc.), not an empty block.
            const HASH_KEYWORDS: &[&[u8]] = &[
                b"return", b"rescue", b"yield", b"raise", b"break", b"next",
                b"then", b"else", b"elsif", b"when", b"do", b"begin",
                b"if", b"unless", b"while", b"until", b"in", b"and", b"or", b"not",
            ];

            let mut in_string: Option<u8> = None;
            while j < len {
                let b = bytes[j];
                // Track string literals so we don't flag `{}` inside strings
                if let Some(delim) = in_string {
                    if b == b'\\' { j += 2; continue; }
                    if b == delim { in_string = None; }
                    j += 1;
                    continue;
                }
                if b == b'"' || b == b'\'' {
                    in_string = Some(b);
                    j += 1;
                    continue;
                }
                if b == b'#' { break; } // comment

                if bytes[j] == b'{' {
                    let open_pos = j;
                    j += 1;
                    // Skip spaces
                    while j < len && bytes[j] == b' ' { j += 1; }
                    // Check if we hit `}`
                    if j < len && bytes[j] == b'}' {
                        // Before flagging, determine if `{` is a block or a hash literal.
                        // Scan backward from open_pos to find the preceding non-space char.
                        let mut k = open_pos as isize - 1;
                        while k >= 0 && bytes[k as usize] == b' ' { k -= 1; }

                        let is_block = if k < 0 {
                            // `{}` at start of line → hash literal
                            false
                        } else {
                            let prev = bytes[k as usize];
                            if prev == b']' {
                                // After array subscript `arr[] {}` → block
                                true
                            } else if prev == b')' {
                                // After args `foo(x) {}` → block, UNLESS it's a lambda `->(x) {}`
                                // Scan backward to find matching `(` and check if `->` precedes it
                                let mut paren_depth = 1i32;
                                let mut m = k - 1;
                                while m >= 0 && paren_depth > 0 {
                                    let c = bytes[m as usize];
                                    if c == b')' { paren_depth += 1; }
                                    else if c == b'(' { paren_depth -= 1; }
                                    m -= 1;
                                }
                                // m now points to char before `(`, skip spaces
                                while m >= 0 && bytes[m as usize] == b' ' { m -= 1; }
                                // If `->` precedes the arg list, it's a lambda body → not a block
                                let is_lambda = m >= 1
                                    && bytes[m as usize] == b'>'
                                    && bytes[(m - 1) as usize] == b'-';
                                !is_lambda
                            } else if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'?' {
                                // Word char: extract word and check if it's a keyword
                                let word_end = k as usize;
                                let mut word_start = word_end;
                                while word_start > 0
                                    && (bytes[word_start - 1].is_ascii_alphanumeric() || bytes[word_start - 1] == b'_')
                                {
                                    word_start -= 1;
                                }
                                let word = &bytes[word_start..=word_end];
                                // Standalone `?` is the ternary operator — `{}` after it is a hash
                                if word == b"?" {
                                    false
                                } else {
                                    !HASH_KEYWORDS.contains(&word)
                                }
                            } else {
                                // Preceded by operator/punctuation (`=`, `|`, `(`, `,`, etc.) → hash
                                false
                            }
                        };

                        if is_block {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Empty block detected.".into(),
                                range: TextRange::new((line_start + open_pos) as u32, (line_start + j + 1) as u32),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    continue;
                }
                j += 1;
            }

            // Also detect `do\nend` (empty do..end block)
            // Check if line ends with `do` and next non-blank line is `end`
            let trimmed_end = line.trim_end();
            if (trimmed_end.ends_with(" do") || trimmed_end == "do") && i + 1 < n && lines[i + 1].trim() == "end" {
                let line_start = ctx.line_start_offsets[i] as usize;
                let do_pos = trimmed_end.len() - 2;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Empty `do..end` block detected.".into(),
                    range: TextRange::new((line_start + do_pos) as u32, (line_start + do_pos + 2) as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

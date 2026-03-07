use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantSplatExpansion;

/// Returns true when `/` at position `j` in `bytes` starts a regex literal
/// rather than a division operator.
fn slash_starts_regex(bytes: &[u8], j: usize) -> bool {
    let mut k = j;
    loop {
        if k == 0 { return true; }
        k -= 1;
        let b = bytes[k];
        if b == b' ' || b == b'\t' { continue; }
        if b.is_ascii_alphanumeric() || b == b'_' || b == b')' || b == b']' {
            return false;
        }
        return true;
    }
}

impl Rule for RedundantSplatExpansion {
    fn name(&self) -> &'static str {
        "Lint/RedundantSplatExpansion"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let bytes = ctx.source.as_bytes();
        let n = bytes.len();
        let mut i = 0;
        let mut in_string: Option<u8> = None;
        let mut in_heredoc = false;

        while i < n {
            let b = bytes[i];

            // Newline: check for heredoc terminator (simple: any non-indented all-caps word)
            // Just reset heredoc tracking on blank lines as a simplification.
            if b == b'\n' {
                i += 1;
                continue;
            }

            // ── String state ─────────────────────────────────────────────────
            match in_string {
                Some(_) if b == b'\\' => { i += 2; continue; }
                Some(delim) if b == delim => { in_string = None; i += 1; continue; }
                Some(_) => { i += 1; continue; }
                None if b == b'"' || b == b'\'' || b == b'`' => {
                    in_string = Some(b);
                    i += 1;
                    continue;
                }
                None if b == b'#' => {
                    // Inline comment — skip to end of line
                    while i < n && bytes[i] != b'\n' { i += 1; }
                    continue;
                }
                None => {}
            }

            // Skip percent literals: %w[...], %r{...}, %(...), etc.
            if b == b'%' && i + 1 < n {
                let next = bytes[i + 1];
                let (has_type, delim_offset) = match next {
                    b'r' | b'w' | b'W' | b'i' | b'I' | b'q' | b'Q' | b'x' => (true, i + 2),
                    b'(' | b'[' | b'{' | b'|' => (false, i + 1),
                    _ => (false, usize::MAX),
                };
                if delim_offset < n {
                    let open = bytes[delim_offset];
                    let close = match open {
                        b'(' => b')', b'[' => b']', b'{' => b'}', b'<' => b'>',
                        _ => open,
                    };
                    let mut j = delim_offset + 1;
                    if open == close {
                        while j < n && bytes[j] != close { if bytes[j] == b'\\' { j += 2; } else { j += 1; } }
                        if j < n { j += 1; }
                    } else {
                        let mut depth = 1usize;
                        while j < n && depth > 0 {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                c if c == open => { depth += 1; j += 1; }
                                c if c == close => { depth -= 1; j += 1; }
                                _ => { j += 1; }
                            }
                        }
                    }
                    i = j;
                    let _ = has_type;
                    continue;
                }
            }

            // ── Skip regex literals: /pattern/ ───────────────────────────────
            if b == b'/' && slash_starts_regex(bytes, i) {
                i += 1; // skip opening `/`
                while i < n && bytes[i] != b'/' && bytes[i] != b'\n' {
                    if bytes[i] == b'\\' { i += 2; continue; }
                    if bytes[i] == b'[' {
                        // character class — skip until `]`
                        i += 1;
                        while i < n && bytes[i] != b']' && bytes[i] != b'\n' {
                            if bytes[i] == b'\\' { i += 2; continue; }
                            i += 1;
                        }
                        if i < n { i += 1; } // skip `]`
                        continue;
                    }
                    i += 1;
                }
                if i < n { i += 1; } // skip closing `/`
                continue;
            }

            // ── Detect `*[` ──────────────────────────────────────────────────
            if b == b'*' && i + 1 < n && bytes[i + 1] == b'[' {
                // Find the matching `]` for this `[`
                let bracket_start = i + 1;
                let mut j = bracket_start + 1;
                let mut depth = 1usize;
                while j < n && depth > 0 {
                    match bytes[j] {
                        b'\\' => { j += 2; }
                        b'[' => { depth += 1; j += 1; }
                        b']' => { depth -= 1; j += 1; }
                        b'\n' => { break; } // Unclosed bracket — give up
                        _ => { j += 1; }
                    }
                }

                if depth == 0 {
                    // `j` is now just past the closing `]`
                    // Skip if followed by `.` — method call on array (not a plain literal)
                    if j < n && bytes[j] == b'.' {
                        i += 1;
                        continue;
                    }
                    // Skip if followed by `[` — subscript access
                    if j < n && bytes[j] == b'[' {
                        i += 1;
                        continue;
                    }
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Redundant splat expansion on array literal `*[...]`.".into(),
                        range: TextRange::new(i as u32, (i + 2) as u32),
                        severity: Severity::Warning,
                    });
                }
                i += 1;
                continue;
            }

            i += 1;
        }
        let _ = in_heredoc;

        diags
    }
}

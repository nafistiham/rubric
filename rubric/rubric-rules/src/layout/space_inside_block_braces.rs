use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceInsideBlockBraces;

// Describes what kind of brace `{` opened.
#[derive(Clone, Copy)]
enum BraceKind {
    Hash,
    // Block opened with a space or other non-pipe content after `{`.
    Block,
    // Block opened in "tight" style: `{` directly follows a word character (no
    // preceding space) AND is immediately followed by `|` (the block-param
    // delimiter). In this style RuboCop does not require interior spaces, so
    // both the open and close brace checks are suppressed.
    TightPipeBlock,
}

fn extract_heredoc_terminator_sibb(line: &[u8]) -> Option<Vec<u8>> {
    let n = line.len();
    let mut i = 0;
    while i + 1 < n {
        if line[i] == b'<' && line[i + 1] == b'<' {
            i += 2;
            if i < n && (line[i] == b'-' || line[i] == b'~') { i += 1; }
            if i < n && (line[i] == b'\'' || line[i] == b'"' || line[i] == b'`') { i += 1; }
            let start = i;
            while i < n && (line[i].is_ascii_alphanumeric() || line[i] == b'_') { i += 1; }
            if i > start { return Some(line[start..i].to_vec()); }
        } else { i += 1; }
    }
    None
}

impl Rule for SpaceInsideBlockBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideBlockBraces"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc: Option<Vec<u8>> = None;
        let mut in_multiline_regex = false;
        // Cross-line percent literal: Some((close_byte, depth))
        let mut in_multiline_percent: Option<(u8, usize)> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // ── Heredoc body ─────────────────────────────────────────────────
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                continue;
            }

            // ── Multiline percent literal body ───────────────────────────────
            if let Some((close, depth)) = in_multiline_percent {
                let bytes = line.as_bytes();
                let nb = bytes.len();
                let new_state = if depth == 0 {
                    let mut k = 0;
                    let mut found = false;
                    while k < nb {
                        match bytes[k] {
                            b'\\' => { k += 2; }
                            c if c == close => { found = true; break; }
                            _ => { k += 1; }
                        }
                    }
                    if found { None } else { Some((close, 0usize)) }
                } else {
                    let open = match close {
                        b')' => b'(', b']' => b'[', b'}' => b'{', b'>' => b'<', _ => 0,
                    };
                    let mut d = depth;
                    let mut k = 0;
                    while k < nb && d > 0 {
                        match bytes[k] {
                            b'\\' => { k += 2; }
                            c if open != 0 && c == open => { d += 1; k += 1; }
                            c if c == close => { d -= 1; k += 1; }
                            _ => { k += 1; }
                        }
                    }
                    if d == 0 { None } else { Some((close, d)) }
                };
                in_multiline_percent = new_state;
                continue;
            }

            // ── Multiline /regex/ body ────────────────────────────────────────
            if in_multiline_regex {
                let bytes = line.as_bytes();
                let n = bytes.len();
                let mut k = 0;
                let mut closed = false;
                while k < n {
                    match bytes[k] {
                        b'\\' => { k += 2; }
                        b'/' => { closed = true; k += 1; break; }
                        _ => { k += 1; }
                    }
                }
                if closed {
                    in_multiline_regex = false;
                }
                continue;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let n = bytes.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            let mut pos = 0;
            let mut in_string: Option<u8> = None;
            // Stack recording what kind of brace each `{` opened.
            // Used to give the `}` check the same context as the `{` check.
            let mut brace_kind_stack: Vec<BraceKind> = Vec::new();

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

                // Skip percent literals (any form: %r, %w, %x, %q, %Q, %{, etc.)
                if b == b'%' && pos + 1 < n {
                    let mut k = pos + 1;
                    // Optional type letter
                    if k < n && bytes[k].is_ascii_alphabetic() { k += 1; }
                    if k < n {
                        let open = bytes[k];
                        let (close, is_bracket) = match open {
                            b'{' => (b'}', true),
                            b'(' => (b')', true),
                            b'[' => (b']', true),
                            b'<' => (b'>', true),
                            b if b.is_ascii_punctuation() => (b, false),
                            _ => { pos += 1; continue; }
                        };
                        pos = k + 1;
                        if is_bracket {
                            let mut depth = 1usize;
                            while pos < n && depth > 0 {
                                match bytes[pos] {
                                    b'\\' => { pos += 2; }
                                    c if c == open => { depth += 1; pos += 1; }
                                    c if c == close => { depth -= 1; pos += 1; }
                                    _ => { pos += 1; }
                                }
                            }
                            if depth > 0 { in_multiline_percent = Some((close, depth)); }
                        } else {
                            let mut closed = false;
                            while pos < n {
                                if bytes[pos] == b'\\' { pos += 2; continue; }
                                if bytes[pos] == close { pos += 1; closed = true; break; }
                                pos += 1;
                            }
                            if !closed { in_multiline_percent = Some((close, 0)); }
                        }
                        continue;
                    }
                    pos += 1;
                    continue;
                }

                // Skip /regex/ literals; detect unclosed regex that spans to the next line.
                if b == b'/' {
                    let prev = if pos > 0 { bytes[pos - 1] } else { 0 };
                    if prev == b'=' || prev == b'(' || prev == b','
                        || prev == b'[' || prev == b' ' || prev == b'\t' || prev == 0
                    {
                        pos += 1;
                        let mut closed = false;
                        while pos < n {
                            match bytes[pos] {
                                b'\\' => { pos += 2; }
                                b'/' => { closed = true; pos += 1; break; }
                                _ => { pos += 1; }
                            }
                        }
                        if !closed {
                            in_multiline_regex = true;
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

                    // The character immediately before `{` (no whitespace skipping).
                    let prev_immediate = if pos > 0 { bytes[pos - 1] } else { 0 };

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

                    // Detect "tight pipe-params" style: `find{|i| ...}`.
                    // Conditions: it is a block (not a hash), `{` is immediately
                    // followed by `|`, AND there is no space immediately before `{`
                    // (prev_immediate is a word char, `)`, or `]`). In this style
                    // the writer has opted out of interior spaces, so we suppress
                    // both the open-brace and close-brace checks.
                    let is_tight_pipe = !is_hash
                        && next == b'|'
                        && (prev_immediate.is_ascii_alphanumeric()
                            || prev_immediate == b'_'
                            || prev_immediate == b')'
                            || prev_immediate == b']');

                    let kind = if is_hash {
                        BraceKind::Hash
                    } else if is_tight_pipe {
                        BraceKind::TightPipeBlock
                    } else {
                        BraceKind::Block
                    };
                    brace_kind_stack.push(kind);

                    if matches!(kind, BraceKind::Block)
                        && next != b' ' && next != b'\n' && next != b'}' && next != 0
                    {
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
                    // Only fire if this `}` closes a plain block brace (not a hash or
                    // a tight-pipe-params block).
                    let kind = brace_kind_stack.pop().unwrap_or(BraceKind::Hash);
                    if matches!(kind, BraceKind::Block) && pos > 0 {
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

            // Detect heredoc opening (body starts on the NEXT line)
            if in_heredoc.is_none() {
                if let Some(term) = extract_heredoc_terminator_sibb(bytes) {
                    in_heredoc = Some(term);
                }
            }
        }

        diags
    }
}

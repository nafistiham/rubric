use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceBeforeBlockBraces;

fn extract_heredoc_terminator_sbbb(line: &[u8]) -> Option<Vec<u8>> {
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

impl Rule for SpaceBeforeBlockBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeBlockBraces"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        let mut in_heredoc: Option<Vec<u8>> = None;
        let mut in_multiline_regex = false;
        // Cross-line percent literal: Some((close_byte, depth))
        // depth == 0 for same-char delimiters, >= 1 for bracket-style
        let mut in_multiline_percent: Option<(u8, usize)> = None;

        for (i, line) in lines.iter().enumerate() {
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
                let mut k = 0;
                while k < bytes.len() {
                    match bytes[k] {
                        b'\\' => { k += 2; }
                        b'/' => { in_multiline_regex = false; break; }
                        _ => { k += 1; }
                    }
                }
                continue;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let len = bytes.len();
            let line_start = ctx.line_start_offsets[i] as usize;
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

                // Skip percent literals (any form: %r, %w, %x, %q, %Q, %{, etc.)
                if b == b'%' && j + 1 < len {
                    let mut k = j + 1;
                    // Optional type letter
                    if k < len && bytes[k].is_ascii_alphabetic() { k += 1; }
                    if k < len {
                        let open = bytes[k];
                        let (close, is_bracket) = match open {
                            b'{' => (b'}', true),
                            b'(' => (b')', true),
                            b'[' => (b']', true),
                            b'<' => (b'>', true),
                            b if b.is_ascii_punctuation() => (b, false),
                            _ => { j += 1; continue; }
                        };
                        j = k + 1;
                        if is_bracket {
                            let mut depth = 1usize;
                            while j < len && depth > 0 {
                                match bytes[j] {
                                    b'\\' => { j += 2; }
                                    c if c == open => { depth += 1; j += 1; }
                                    c if c == close => { depth -= 1; j += 1; }
                                    _ => { j += 1; }
                                }
                            }
                            if depth > 0 { in_multiline_percent = Some((close, depth)); }
                        } else {
                            let mut closed = false;
                            while j < len {
                                if bytes[j] == b'\\' { j += 2; continue; }
                                if bytes[j] == close { j += 1; closed = true; break; }
                                j += 1;
                            }
                            if !closed { in_multiline_percent = Some((close, 0)); }
                        }
                        continue;
                    }
                    j += 1;
                    continue;
                }

                // Skip /regex/ literals; detect unclosed multiline regex
                if b == b'/' {
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    if prev == b'=' || prev == b'(' || prev == b','
                        || prev == b'[' || prev == b' ' || prev == b'\t' || prev == 0
                    {
                        j += 1;
                        let mut closed = false;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'/' => { closed = true; j += 1; break; }
                                _ => { j += 1; }
                            }
                        }
                        if !closed { in_multiline_regex = true; }
                        continue;
                    }
                }

                // Check for word char immediately followed by `{`
                if b == b'{' && j > 0 {
                    let prev = bytes[j - 1];
                    if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b')' || prev == b']' {
                        let brace_pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Missing space before block `{`.".into(),
                            range: TextRange::new(brace_pos, brace_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }
                j += 1;
            }

            // Detect heredoc opening (body starts on the NEXT line)
            if in_heredoc.is_none() {
                if let Some(term) = extract_heredoc_terminator_sbbb(bytes) {
                    in_heredoc = Some(term);
                }
            }
        }

        diags
    }
}

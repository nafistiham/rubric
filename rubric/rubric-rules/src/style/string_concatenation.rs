use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct StringConcatenation;

/// Extract heredoc terminator from a line.
fn heredoc_terminator(line: &str) -> Option<String> {
    let marker = if let Some(p) = line.find("<<~") {
        Some(&line[p + 3..])
    } else if let Some(p) = line.find("<<-") {
        Some(&line[p + 3..])
    } else if let Some(p) = line.find("<<") {
        let after = &line[p + 2..];
        if after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '"' || c == '\'') {
            Some(after)
        } else {
            None
        }
    } else {
        None
    }?;
    let rest = marker.trim_start_matches(|c: char| c == '\'' || c == '"');
    let term: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if term.is_empty() { None } else { Some(term) }
}

impl Rule for StringConcatenation {
    fn name(&self) -> &'static str {
        "Style/StringConcatenation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Heredoc body-skip: lines inside heredocs are string content, not code.
        let mut in_heredoc = false;
        let mut heredoc_term = String::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            // ── Heredoc body ─────────────────────────────────────────────────
            if in_heredoc {
                if line.trim() == heredoc_term.as_str() {
                    in_heredoc = false;
                    heredoc_term.clear();
                }
                continue;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Look for `" + "` or `' + '` patterns (BOTH sides must be string literals).
            // Rubocop only flags string + string concatenation; it does NOT flag
            // `str + variable` or `str + method_call` — only literal + literal.
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut pos = 0;
            let mut in_string: Option<u8> = None;

            while pos < n {
                let b = bytes[pos];
                match in_string {
                    Some(_) if b == b'\\' => { pos += 2; continue; }
                    Some(delim) if b == delim => {
                        // String just ended — check if followed by ` + "` or ` + '`
                        in_string = None;
                        pos += 1;
                        // Skip whitespace
                        let mut j = pos;
                        while j < n && bytes[j] == b' ' { j += 1; }
                        if j < n && bytes[j] == b'+' {
                            // Check it's not `+=`
                            let after_plus = if j + 1 < n { bytes[j + 1] } else { 0 };
                            if after_plus != b'=' {
                                // RHS must also be a string literal — skip whitespace after
                                // `+` and verify the next char is a quote.
                                let mut k = j + 1;
                                while k < n && bytes[k] == b' ' { k += 1; }
                                if k < n && (bytes[k] == b'"' || bytes[k] == b'\'') {
                                    let line_start = ctx.line_start_offsets[i] as usize;
                                    let flag_pos = (line_start + j) as u32;
                                    diags.push(Diagnostic {
                                        rule: self.name(),
                                        message: "Use string interpolation instead of string concatenation.".into(),
                                        range: TextRange::new(flag_pos, flag_pos + 1),
                                        severity: Severity::Warning,
                                    });
                                }
                            }
                        }
                        continue;
                    }
                    Some(_) => { pos += 1; continue; }
                    None if b == b'"' || b == b'\'' => {
                        in_string = Some(b);
                        pos += 1;
                        continue;
                    }
                    None if b == b'%' && pos + 1 < n => {
                        // Skip percent literals: %(str), %q(...), %w[...], etc.
                        let mut k = pos + 1;
                        if k < n && bytes[k].is_ascii_alphabetic() { k += 1; }
                        if k < n {
                            let open = bytes[k];
                            let close = match open {
                                b'(' => b')', b'[' => b']', b'{' => b'}', b'<' => b'>',
                                c if c.is_ascii_punctuation() => c,
                                _ => { pos += 1; continue; }
                            };
                            k += 1;
                            if open == close {
                                while k < n { if bytes[k] == b'\\' { k += 2; continue; } if bytes[k] == close { k += 1; break; } k += 1; }
                            } else {
                                let mut depth = 1usize;
                                while k < n && depth > 0 { if bytes[k] == b'\\' { k += 2; continue; } if bytes[k] == open { depth += 1; } else if bytes[k] == close { depth -= 1; } k += 1; }
                            }
                            pos = k;
                        } else {
                            pos += 1;
                        }
                        continue;
                    }
                    None if b == b'#' => break,
                    None => {}
                }
                pos += 1;
            }

            // Enter heredoc mode for subsequent lines.
            if let Some(term) = heredoc_terminator(line) {
                in_heredoc = true;
                heredoc_term = term;
            }
        }
        diags
    }
}

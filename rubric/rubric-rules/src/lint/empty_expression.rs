use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyExpression;

fn extract_heredoc_terminator(line: &[u8]) -> Option<String> {
    let len = line.len();
    let mut i = 0;
    while i + 1 < len {
        if line[i] == b'<' && line[i + 1] == b'<' {
            let mut k = i + 2;
            if k < len && (line[k] == b'-' || line[k] == b'~') {
                k += 1;
            }
            let quote = if k < len && (line[k] == b'\'' || line[k] == b'"' || line[k] == b'`') {
                let q = line[k];
                k += 1;
                Some(q)
            } else {
                None
            };
            let term_start = k;
            if let Some(q) = quote {
                while k < len && line[k] != q {
                    k += 1;
                }
            } else {
                while k < len && (line[k].is_ascii_alphanumeric() || line[k] == b'_') {
                    k += 1;
                }
            }
            if k > term_start {
                return Some(String::from_utf8_lossy(&line[term_start..k]).to_string());
            }
        }
        i += 1;
    }
    None
}

impl Rule for EmptyExpression {
    fn name(&self) -> &'static str {
        "Lint/EmptyExpression"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        let mut in_heredoc: Option<String> = None;
        let mut in_multiline_regex = false;
        let mut in_percent_regex = false;
        let mut percent_regex_depth = 0usize;

        for (i, line) in lines.iter().enumerate() {
            let bytes = line.as_bytes();
            let len = bytes.len();

            // --- Heredoc body ---
            if let Some(ref term) = in_heredoc.clone() {
                let stripped = line.trim();
                if stripped == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // --- Multiline /regex/ body ---
            if in_multiline_regex {
                let mut j = 0;
                while j < len {
                    match bytes[j] {
                        b'\\' => { j += 2; continue; }
                        b'/' => { in_multiline_regex = false; break; }
                        _ => { j += 1; }
                    }
                }
                continue;
            }

            // --- Multiline %r{...} body ---
            if in_percent_regex {
                let mut j = 0;
                while j < len {
                    match bytes[j] {
                        b'\\' => { j += 2; continue; }
                        b'{' => { percent_regex_depth += 1; j += 1; }
                        b'}' => {
                            if percent_regex_depth > 0 {
                                percent_regex_depth -= 1;
                                j += 1;
                            } else {
                                in_percent_regex = false;
                                break;
                            }
                        }
                        _ => { j += 1; }
                    }
                }
                continue;
            }

            // --- Normal line ---
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Check for heredoc start on this line (next lines will be body)
            if let Some(term) = extract_heredoc_terminator(bytes) {
                in_heredoc = Some(term);
            }

            let mut j = 0;
            let mut in_string: Option<u8> = None;

            while j < len {
                let b = bytes[j];

                // String tracking
                if let Some(delim) = in_string {
                    if b == b'\\' { j += 2; continue; }
                    if b == delim { in_string = None; }
                    j += 1;
                    continue;
                }

                // Comment
                if b == b'#' { break; }

                // String start
                if b == b'"' || b == b'\'' {
                    in_string = Some(b);
                    j += 1;
                    continue;
                }

                // Percent regex %r{...} — skip the body
                if b == b'%' && j + 1 < len && bytes[j + 1] == b'r' {
                    j += 2;
                    if j < len && bytes[j] == b'{' {
                        j += 1;
                        let mut depth = 1usize;
                        let mut closed = false;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; continue; }
                                b'{' => { depth += 1; j += 1; }
                                b'}' => {
                                    depth -= 1;
                                    j += 1;
                                    if depth == 0 { closed = true; break; }
                                }
                                _ => { j += 1; }
                            }
                        }
                        if !closed {
                            in_percent_regex = true;
                            percent_regex_depth = 0;
                        }
                    } else if j < len {
                        let delim = bytes[j];
                        j += 1;
                        while j < len && bytes[j] != delim {
                            if bytes[j] == b'\\' { j += 2; } else { j += 1; }
                        }
                        if j < len { j += 1; }
                    }
                    continue;
                }

                // /regex/ literal — skip the body (including char classes)
                if b == b'/' {
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    let is_regex_start = prev == b'=' || prev == b'(' || prev == b','
                        || prev == b'[' || prev == b' ' || prev == b'\t' || j == 0;
                    if is_regex_start {
                        j += 1;
                        let mut closed = false;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; continue; }
                                b'[' => {
                                    // character class — skip to ]
                                    j += 1;
                                    while j < len && bytes[j] != b']' {
                                        if bytes[j] == b'\\' { j += 2; } else { j += 1; }
                                    }
                                    if j < len { j += 1; }
                                }
                                b'/' => {
                                    closed = true;
                                    j += 1;
                                    // skip modifiers
                                    while j < len && bytes[j].is_ascii_alphabetic() { j += 1; }
                                    break;
                                }
                                _ => { j += 1; }
                            }
                        }
                        if !closed {
                            in_multiline_regex = true;
                        }
                        continue;
                    }
                }

                // EmptyExpression check
                if b == b'(' {
                    let open_pos = j;
                    j += 1;
                    while j < len && (bytes[j] == b' ' || bytes[j] == b'\t') { j += 1; }
                    if j < len && bytes[j] == b')' {
                        let preceded_by_word = open_pos > 0
                            && (bytes[open_pos - 1].is_ascii_alphanumeric()
                                || bytes[open_pos - 1] == b'_');
                        if !preceded_by_word {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Empty expression `()` has no value.".into(),
                                range: TextRange::new(
                                    (line_start + open_pos) as u32,
                                    (line_start + j + 1) as u32,
                                ),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    continue;
                }

                j += 1;
            }
        }

        diags
    }
}

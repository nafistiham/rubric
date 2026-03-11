use rubric_core::{Diagnostic, Fix, LintContext, Rule, Severity, TextRange};

pub struct HashSyntax;

/// Returns true if `line` contains at least one `key => value` pair where the key
/// is not a plain symbol literal (i.e., does not start with `:`).  This indicates
/// a hash with mixed key styles; RuboCop's `ruby19_no_mixed_keys` style leaves such
/// hashes alone, so we must not flag the symbol keys in them either.
fn has_non_symbol_rocket_key(line: &str) -> bool {
    let bytes = line.as_bytes();
    let len = bytes.len();
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

        // Look for `=>` outside strings
        if b == b'=' && j + 1 < len && bytes[j + 1] == b'>' {
            // Scan backwards to find the key token.
            // Skip spaces before `=>`
            let mut k = j;
            while k > 0 && bytes[k - 1] == b' ' {
                k -= 1;
            }
            // The character just before the `=>` (after spaces) tells us the key type.
            // If the key ends with an alphanumeric/`_`/`]`/`)` character, scan back
            // to find the start of the key token.
            if k > 0 {
                let key_last_byte = bytes[k - 1];
                // If the key ends with a quote, it's a string key — non-symbol.
                if key_last_byte == b'"' || key_last_byte == b'\'' || key_last_byte == b'`' {
                    return true;
                }
                // If the key ends with `]` or `)` it's an expression — non-symbol.
                if key_last_byte == b']' || key_last_byte == b')' {
                    return true;
                }
                let key_end = k - 1; // last byte of key token
                // Walk back over the key token
                let mut key_start = key_end;
                while key_start > 0
                    && (bytes[key_start - 1].is_ascii_alphanumeric()
                        || bytes[key_start - 1] == b'_'
                        || bytes[key_start - 1] == b'?'
                        || bytes[key_start - 1] == b'!')
                {
                    key_start -= 1;
                }
                // If the key token is preceded by `::` it's a constant qualifier — the
                // real key starts further back, skip this heuristic for that position.
                // We only care whether the key starts with `:` (symbol) or not.
                let key_byte = if key_start > 0 { bytes[key_start - 1] } else { 0 };
                // key_byte is the character immediately before the word token.
                // If it is `:` and not `::`, the key is a symbol literal.
                let preceded_by_colon = key_byte == b':';
                let preceded_by_double_colon =
                    preceded_by_colon && key_start >= 2 && bytes[key_start - 2] == b':';

                if !preceded_by_colon || preceded_by_double_colon {
                    // The key is NOT a plain `:symbol` — it is a string, constant,
                    // variable, ivar, or namespace-qualified constant.
                    return true;
                }
            }
            j += 2;
            continue;
        }

        j += 1;
    }
    false
}

/// Returns true if any of the next `max_look` lines (starting at `start`)
/// contains a non-symbol rocket key — indicating that the current symbol-rocket
/// line is a continuation of a mixed-key hash/call.
fn lookahead_has_mixed_key(lines: &Vec<&str>, start: usize, max_look: usize) -> bool {
    let end = (start + max_look).min(lines.len());
    for line in &lines[start..end] {
        let trimmed = line.trim();
        // Stop if we hit a line that looks like a new statement (method def, class, etc.)
        if trimmed.starts_with("def ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("module ")
        {
            break;
        }
        if has_non_symbol_rocket_key(line) {
            return true;
        }
        // Stop if line ends with a closing delimiter without continuation comma
        let code_end = line.trim_end();
        if code_end.ends_with('}') || code_end.ends_with(')') {
            // Might close the hash; stop looking
            break;
        }
    }
    false
}

impl Rule for HashSyntax {
    fn name(&self) -> &'static str {
        "Style/HashSyntax"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // Track whether we're in a multi-line argument list that started with a
        // non-symbol rocket key. When true, continuation lines that only contain
        // symbol rocket keys are part of a mixed-key call and must be skipped.
        let mut in_mixed_key_continuation = false;
        // Heredoc tracking: skip body lines
        let mut in_heredoc: Option<Vec<u8>> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                continue;
            }
            // Detect heredoc opener
            {
                let bytes = line.as_bytes();
                let mut k = 0;
                while k + 1 < bytes.len() {
                    if bytes[k] == b'<' && bytes[k + 1] == b'<' {
                        let mut j = k + 2;
                        if j < bytes.len() && (bytes[j] == b'-' || bytes[j] == b'~') { j += 1; }
                        if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`') { j += 1; }
                        let start = j;
                        while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
                        if j > start { in_heredoc = Some(bytes[start..j].to_vec()); break; }
                    }
                    k += 1;
                }
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            // Skip lines that contain a mix of symbol and non-symbol rocket keys.
            // RuboCop's `ruby19_no_mixed_keys` enforcement leaves such hashes alone.
            if has_non_symbol_rocket_key(line) {
                // If this line ends with `,` (open argument list), set continuation.
                let trimmed_end = line.trim_end();
                let code_end = {
                    // Strip trailing inline comment
                    let mut end = trimmed_end;
                    if let Some(p) = trimmed_end.find(" #") { end = &trimmed_end[..p]; }
                    end.trim_end()
                };
                in_mixed_key_continuation = code_end.ends_with(',')
                    || code_end.ends_with('(')
                    || code_end.ends_with("\\");
                continue;
            }

            // If we're in a mixed-key continuation from the previous line, skip
            // symbol-rocket violations on this line. Update continuation state.
            if in_mixed_key_continuation {
                let trimmed_end = line.trim_end();
                let code_end = {
                    let mut end = trimmed_end;
                    if let Some(p) = trimmed_end.find(" #") { end = &trimmed_end[..p]; }
                    end.trim_end()
                };
                // Continuation continues if line ends with `,` or `(` or `\`
                in_mixed_key_continuation = code_end.ends_with(',')
                    || code_end.ends_with('(')
                    || code_end.ends_with("\\");
                continue;
            }

            // Update continuation state for lines with ONLY symbol rocket keys.
            // If such a line ends with `,`, the argument list continues but it's
            // purely symbol keys — continuation flag stays false.
            // (We only set in_mixed_key_continuation above when a non-symbol key line ends with `,`.)
            // After processing the loop body, check if this line is a "pure symbol"
            // rocket line ending with `,` — in that case, continuation stays false.

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

                // Look for `:symbol =>` pattern: starts with `:` followed by word chars then ` =>`
                // Skip `::` namespace separators — the second `:` of `::` is not a symbol literal.
                // E.g. `ActiveRecord::RecordInvalid => e` must not be flagged.
                if b == b':' {
                    if j > 0 && bytes[j - 1] == b':' {
                        j += 1;
                        continue;
                    }
                    // Read the symbol name (word chars)
                    let mut k = j + 1;
                    while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                        k += 1;
                    }

                    // Check if symbol name is non-empty and followed by ` =>`
                    if k > j + 1 {
                        // Skip spaces
                        let mut m = k;
                        while m < len && bytes[m] == b' ' {
                            m += 1;
                        }
                        if m + 1 < len && bytes[m] == b'=' && bytes[m+1] == b'>' {
                            // Before flagging, look ahead for non-symbol rocket keys in
                            // subsequent lines of the same hash (handles mixed-key hashes
                            // where symbol keys appear before string keys).
                            let line_ends_with_comma = line.trim_end().trim_end_matches(|c: char| c == '#' || c.is_alphanumeric() || c == '"' || c == '\'' || c == '_' || c == ' ').is_empty()
                                || {
                                    let te = line.trim_end();
                                    // Strip trailing comment
                                    let code = if let Some(p) = te.rfind(" #") { &te[..p] } else { te };
                                    code.trim_end().ends_with(',')
                                };
                            if line_ends_with_comma && lookahead_has_mixed_key(&ctx.lines, i + 1, 10) {
                                // This is a symbol-rocket key in a mixed-key hash — skip.
                                j = m + 2;
                                continue;
                            }
                            let pos = (line_start + j) as u32;
                            let end = (line_start + m + 2) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Use the new hash syntax (`key: value`) instead of `{:key => value}`.".into(),
                                range: TextRange::new(pos, end),
                                severity: Severity::Warning,
                            });
                            j = m + 2;
                            continue;
                        }
                    }
                }

                j += 1;
            }
        }

        diags
    }

    fn fix(&self, _diag: &Diagnostic) -> Option<Fix> {
        None // TODO: implement :sym => val -> sym: val transformation
    }
}

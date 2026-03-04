use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashMap;

pub struct UselessAssignment;

/// Returns true if the trimmed line contains `keyword` as a standalone word token.
fn line_contains_keyword(line: &str, keyword: &str) -> bool {
    let lb = line.as_bytes();
    let kb = keyword.as_bytes();
    let n = lb.len();
    let kn = kb.len();
    if kn == 0 || n < kn { return false; }
    let mut i = 0;
    while i + kn <= n {
        if &lb[i..i+kn] == kb {
            let before_ok = i == 0 || (!lb[i-1].is_ascii_alphanumeric() && lb[i-1] != b'_');
            let after_ok = i + kn >= n || (!lb[i+kn].is_ascii_alphanumeric() && lb[i+kn] != b'_');
            if before_ok && after_ok { return true; }
        }
        i += 1;
    }
    false
}

/// Returns true if the trimmed line opens a new block-level construct.
/// Checks for openers anywhere in the line (not just at the start) so that
/// patterns like `var = if cond` and `var = begin` increment the depth counter.
fn line_opens_block(t: &str) -> bool {
    if t.starts_with("def ") || t == "def" { return true; }
    if line_contains_keyword(t, "if")     { return true; }
    if line_contains_keyword(t, "unless") { return true; }
    if line_contains_keyword(t, "begin")  { return true; }
    if line_contains_keyword(t, "case")   { return true; }
    if line_contains_keyword(t, "while")  { return true; }
    if line_contains_keyword(t, "until")  { return true; }
    if line_contains_keyword(t, "do")     { return true; }
    if t.starts_with("for ")             { return true; }
    if t == "loop" || t.ends_with(" loop") { return true; }
    false
}

/// If `line` contains a heredoc opener (`<<-WORD`, `<<WORD`, `<<~WORD`),
/// return the terminator string (e.g. `"WORD"`). Otherwise return `None`.
fn heredoc_terminator(line: &str) -> Option<String> {
    let lb = line.as_bytes();
    let n = lb.len();
    let mut i = 0;
    while i + 1 < n {
        if lb[i] == b'<' && lb[i+1] == b'<' {
            i += 2;
            if i < n && (lb[i] == b'-' || lb[i] == b'~') {
                i += 1;
            }
            let quote = if i < n && (lb[i] == b'\'' || lb[i] == b'"' || lb[i] == b'`') {
                let q = lb[i];
                i += 1;
                Some(q)
            } else {
                None
            };
            let word_start = i;
            while i < n && (lb[i].is_ascii_alphanumeric() || lb[i] == b'_') {
                i += 1;
            }
            if i > word_start {
                let word = std::str::from_utf8(&lb[word_start..i]).unwrap_or("").to_string();
                if !word.is_empty() {
                    let _ = quote; // closing quote not needed for matching
                    return Some(word);
                }
            }
        }
        i += 1;
    }
    None
}

/// Scan a `#{...}` block (pos is just after `#{`) for a read of `var`.
/// Returns `(found, new_pos)`.
fn scan_interpolation_for_var(
    line_bytes: &[u8],
    mut pos: usize,
    var: &str,
    assign_line: usize,
    assign_col: usize,
    k: usize,
) -> (bool, usize) {
    let line_len = line_bytes.len();
    let vb = var.as_bytes();
    let mut depth = 1usize;
    while pos < line_len && depth > 0 {
        let ib = line_bytes[pos];
        if ib == b'{' { depth += 1; }
        else if ib == b'}' {
            depth -= 1;
            if depth == 0 { pos += 1; break; }
        }
        if pos + vb.len() <= line_len && &line_bytes[pos..pos+vb.len()] == vb {
            let before_ok = pos == 0 || (!line_bytes[pos-1].is_ascii_alphanumeric() && line_bytes[pos-1] != b'_');
            let after_ok = pos + vb.len() >= line_len
                || (!line_bytes[pos+vb.len()].is_ascii_alphanumeric() && line_bytes[pos+vb.len()] != b'_');
            if before_ok && after_ok && !(k == assign_line && pos == assign_col) {
                return (true, pos);
            }
        }
        pos += 1;
    }
    (false, pos)
}

impl Rule for UselessAssignment {
    fn name(&self) -> &'static str {
        "Lint/UselessAssignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();
            if !trimmed.starts_with("def ") && trimmed != "def" {
                i += 1;
                continue;
            }

            // --- Find matching `end` for this def ---
            // Track heredoc bodies to avoid miscounting `end`/openers inside them.
            // Use line_opens_block() to detect openers mid-line (e.g. `var = begin`).
            let def_start = i;
            let mut depth = 1i32;
            let mut j = i + 1;
            let mut heredoc_term: Option<String> = None;

            while j < n && depth > 0 {
                let raw = &lines[j];
                let t = raw.trim();

                if let Some(ref term) = heredoc_term {
                    if t == term.as_str() {
                        heredoc_term = None;
                    }
                    j += 1;
                    continue;
                }

                if let Some(term) = heredoc_terminator(raw) {
                    heredoc_term = Some(term);
                    // opener line still participates in depth counting below
                }

                if line_opens_block(t) {
                    depth += 1;
                }
                if t == "end" || t.starts_with("end #") || t.starts_with("end;") {
                    depth -= 1;
                }

                j += 1;
            }
            let def_end = j;

            // --- Collect assignments inside the def body ---
            let mut assignments: HashMap<String, (usize, usize)> = HashMap::new();
            {
                let mut heredoc_term_asgn: Option<String> = None;
                for k in (def_start + 1)..def_end.min(n) {
                    let raw = &lines[k];
                    let tk = raw.trim();
                    if let Some(ref term) = heredoc_term_asgn {
                        if tk == term.as_str() {
                            heredoc_term_asgn = None;
                        }
                        continue; // skip heredoc body lines
                    }
                    if let Some(term) = heredoc_terminator(raw) {
                        heredoc_term_asgn = Some(term);
                    }

                    let line_bytes = raw.as_bytes();
                    let line_len = line_bytes.len();
                    let mut pos = 0;
                    let mut in_string: Option<u8> = None;

                    while pos < line_len {
                        let b = line_bytes[pos];
                        match in_string {
                            Some(_) if b == b'\\' => { pos += 2; continue; }
                            Some(delim) if b == delim => { in_string = None; pos += 1; continue; }
                            Some(_) => { pos += 1; continue; }
                            None if b == b'"' || b == b'\'' => { in_string = Some(b); pos += 1; continue; }
                            None if b == b'#' => break,
                            None => {}
                        }

                        if b.is_ascii_lowercase() || b == b'_' {
                            let word_start = pos;
                            while pos < line_len
                                && (line_bytes[pos].is_ascii_alphanumeric() || line_bytes[pos] == b'_')
                            {
                                pos += 1;
                            }
                            let word = std::str::from_utf8(&line_bytes[word_start..pos]).unwrap_or("");

                            if matches!(
                                word,
                                "def" | "end" | "if" | "unless" | "else" | "do"
                                    | "return" | "begin" | "case" | "while" | "until"
                            ) {
                                continue;
                            }
                            if word.starts_with('_') {
                                continue;
                            }

                            let mut eq_pos = pos;
                            while eq_pos < line_len && line_bytes[eq_pos] == b' ' {
                                eq_pos += 1;
                            }

                            if eq_pos < line_len && line_bytes[eq_pos] == b'=' {
                                let after_eq = if eq_pos + 1 < line_len { line_bytes[eq_pos+1] } else { 0 };
                                // Exclude `==`, `=>`, `=~` (regex match), `=<`
                                if after_eq != b'=' && after_eq != b'>' && after_eq != b'~' && after_eq != b'<' {
                                    let indent_end = raw.len() - raw.trim_start().len();
                                    if word_start == indent_end {
                                        assignments.insert(word.to_string(), (k, word_start));
                                    }
                                }
                            }
                            continue;
                        }

                        pos += 1;
                    }
                }
            }

            // --- Check which variables are never read ---
            'outer: for (var, (assign_line, assign_col)) in &assignments {
                let mut heredoc_term_usage: Option<String> = None;

                for k in (def_start + 1)..def_end.min(n) {
                    let line = &lines[k];
                    let line_bytes = line.as_bytes();
                    let line_len = line_bytes.len();

                    // Track heredoc state; still scan heredoc body lines for #{} reads
                    {
                        let tk = line.trim();
                        if let Some(ref term) = heredoc_term_usage {
                            if tk == term.as_str() {
                                heredoc_term_usage = None;
                                continue;
                            }
                            // fall through: scan heredoc body for #{} reads
                        }
                        if let Some(term) = heredoc_terminator(line) {
                            heredoc_term_usage = Some(term);
                        }
                    }

                    let mut pos = 0;
                    let mut in_single_string = false;
                    let mut in_double_string = false;
                    // Track last non-whitespace byte to detect regex vs division for `/`
                    let mut last_non_space: u8 = 0;

                    while pos < line_len {
                        let b = line_bytes[pos];

                        if in_single_string {
                            if b == b'\\' { pos += 2; continue; }
                            if b == b'\'' { in_single_string = false; last_non_space = b'\''; }
                            pos += 1;
                            continue;
                        }

                        if in_double_string {
                            if b == b'\\' { pos += 2; continue; }
                            if b == b'"' { in_double_string = false; last_non_space = b'"'; pos += 1; continue; }
                            if b == b'#' {
                                let next = if pos + 1 < line_len { line_bytes[pos+1] } else { 0 };
                                if next != b'{' { pos += 1; continue; }
                                pos += 2;
                                let (found, new_pos) = scan_interpolation_for_var(
                                    line_bytes, pos, var, *assign_line, *assign_col, k,
                                );
                                pos = new_pos;
                                if found { continue 'outer; }
                                continue;
                            }
                            pos += 1;
                            continue;
                        }

                        // Normal context
                        if b == b' ' || b == b'\t' { pos += 1; continue; } // skip space without updating last_non_space
                        if b == b'\'' { in_single_string = true; last_non_space = b; pos += 1; continue; }
                        if b == b'"' { in_double_string = true; last_non_space = b; pos += 1; continue; }
                        if b == b'/' {
                            // Heuristic: `/` is a regex opener when the preceding non-space
                            // char is an operator, open bracket, or start of line.
                            let is_regex_start = matches!(
                                last_non_space,
                                0 | b'(' | b',' | b'[' | b'=' | b'!' | b'+' | b'-'
                                    | b'*' | b'%' | b'|' | b'&' | b'^' | b'~' | b'<' | b'>'
                            );
                            if is_regex_start {
                                // Skip regex literal content (with #{} interpolation support)
                                pos += 1; // move past opening /
                                let mut in_class = false; // inside [...]
                                while pos < line_len {
                                    let rb = line_bytes[pos];
                                    if rb == b'\\' { pos += 2; continue; }
                                    if rb == b'[' { in_class = true; pos += 1; continue; }
                                    if rb == b']' { in_class = false; pos += 1; continue; }
                                    if rb == b'#' {
                                        let rn = if pos + 1 < line_len { line_bytes[pos+1] } else { 0 };
                                        if rn == b'{' {
                                            pos += 2;
                                            let (found, new_pos) = scan_interpolation_for_var(
                                                line_bytes, pos, var, *assign_line, *assign_col, k,
                                            );
                                            pos = new_pos;
                                            if found { continue 'outer; }
                                            continue;
                                        }
                                    }
                                    if rb == b'/' && !in_class {
                                        // closing slash — skip optional flags (i, m, x, etc.)
                                        pos += 1;
                                        while pos < line_len && line_bytes[pos].is_ascii_alphabetic() {
                                            pos += 1;
                                        }
                                        break;
                                    }
                                    pos += 1;
                                }
                                last_non_space = b'/';
                                continue;
                            }
                            last_non_space = b;
                            pos += 1;
                            continue;
                        }
                        if b == b'#' {
                            let next = if pos + 1 < line_len { line_bytes[pos+1] } else { 0 };
                            if next == b'{' {
                                // #{...} outside a string (regex, heredoc body, etc.)
                                pos += 2;
                                let (found, new_pos) = scan_interpolation_for_var(
                                    line_bytes, pos, var, *assign_line, *assign_col, k,
                                );
                                pos = new_pos;
                                if found { continue 'outer; }
                                continue;
                            }
                            break; // inline comment
                        }

                        last_non_space = b;
                        let vb = var.as_bytes();
                        if pos + vb.len() <= line_len && &line_bytes[pos..pos+vb.len()] == vb {
                            let before_ok = pos == 0
                                || (!line_bytes[pos-1].is_ascii_alphanumeric() && line_bytes[pos-1] != b'_');
                            let after_ok = pos + vb.len() >= line_len
                                || (!line_bytes[pos+vb.len()].is_ascii_alphanumeric()
                                    && line_bytes[pos+vb.len()] != b'_');

                            if before_ok && after_ok {
                                let is_lhs = k == *assign_line && pos == *assign_col;
                                if !is_lhs {
                                    continue 'outer;
                                }
                            }
                        }
                        pos += 1;
                    }
                }

                let line_start = ctx.line_start_offsets[*assign_line] as usize;
                let offset = (line_start + assign_col) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!("Useless assignment to variable `{}`.", var),
                    range: TextRange::new(offset, offset + var.len() as u32),
                    severity: Severity::Warning,
                });
            }

            i = def_end;
        }

        diags
    }
}

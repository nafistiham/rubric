use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AndOr;

const FLOW_CONTROL_KEYWORDS: &[&str] = &["raise", "return", "next", "break", "fail"];

fn is_flow_control_rhs(rest: &str) -> bool {
    let word = rest.trim_start();
    FLOW_CONTROL_KEYWORDS.iter().any(|&kw| {
        word.starts_with(kw)
            && word[kw.len()..]
                .chars()
                .next()
                .map_or(true, |c| !c.is_alphanumeric() && c != '_')
    })
}

/// Returns true if `pos` is inside a string literal on `line`.
/// Handles `"..."` and `'...'` with backslash escapes. Stops at `#` outside strings (comment).
fn pos_in_string(line: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos && i < line.len() {
        match in_str {
            Some(_) if line[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(delim) if line[i] == delim => {
                in_str = None;
            }
            Some(_) => {}
            None if line[i] == b'"' || line[i] == b'\'' => {
                in_str = Some(line[i]);
            }
            None if line[i] == b'#' => return false, // comment — nothing after this is code
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

impl Rule for AndOr {
    fn name(&self) -> &'static str {
        "Style/AndOr"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        // Track heredoc state
        let mut in_heredoc: Option<String> = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Handle heredoc end
            if let Some(ref terminator) = in_heredoc.clone() {
                if line.trim() == terminator.as_str() {
                    in_heredoc = None;
                }
                continue; // skip heredoc body lines
            }

            if trimmed.starts_with('#') {
                continue;
            }

            // Detect heredoc openers: <<~TERM, <<-TERM, <<TERM (quoted or bare)
            // Extract the terminator to know when the heredoc ends.
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Don't skip this line entirely — the content after the opener is code
            }

            let bytes = line.as_bytes();

            for (pattern, kw_len) in &[(" and ", 3usize), (" or ", 2usize)] {
                let mut search_start = 0usize;
                while let Some(pos) = line[search_start..].find(pattern) {
                    let abs_pos = search_start + pos;

                    // Skip if this match is inside a string literal
                    if pos_in_string(bytes, abs_pos) {
                        search_start = abs_pos + 1;
                        if search_start >= line.len() {
                            break;
                        }
                        continue;
                    }

                    let after_pattern = &line[abs_pos + pattern.len()..];

                    if is_flow_control_rhs(after_pattern) {
                        search_start = abs_pos + pattern.len();
                        if search_start >= line.len() {
                            break;
                        }
                        continue;
                    }

                    let kw_start = abs_pos + 1;
                    let line_start = ctx.line_start_offsets[i] as usize;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `&&`/`||` instead of `and`/`or` keyword.".to_string(),
                        range: TextRange::new(
                            (line_start + kw_start) as u32,
                            (line_start + kw_start + kw_len) as u32,
                        ),
                        severity: Severity::Warning,
                    });
                    search_start = abs_pos + pattern.len();
                    if search_start >= line.len() {
                        break;
                    }
                }
            }
        }

        diags
    }
}

/// Extract the heredoc terminator from a line containing `<<~TERM`, `<<-TERM`, or `<<TERM`.
/// Returns `None` if no heredoc opener found.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let mut i = 0;
    let bytes = line.as_bytes();
    let len = bytes.len();
    // Track string state to avoid matching << inside strings
    let mut in_str: Option<u8> = None;
    while i + 1 < len {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
                i += 1;
                continue;
            }
            Some(_) => {
                i += 1;
                continue;
            }
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
                i += 1;
                continue;
            }
            None if bytes[i] == b'#' => break,
            None => {}
        }
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            // Skip optional `-` or `~`
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') {
                j += 1;
            }
            // Skip optional quote around terminator
            let quote = if j < len
                && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`')
            {
                let q = bytes[j];
                j += 1;
                Some(q)
            } else {
                None
            };
            let _ = quote; // terminator word is what matters
            // Read the terminator word
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j > start {
                let term = &line[start..j];
                return Some(term.to_string());
            }
        }
        i += 1;
    }
    None
}

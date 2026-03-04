use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineMethodCallBraceLayout;

impl Rule for MultilineMethodCallBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodCallBraceLayout"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track heredoc state: when inside a heredoc body, skip lines.
        let mut heredoc_end: Option<String> = None;

        for i in 1..n {
            let line = &lines[i];
            let trimmed = line.trim();

            // Maintain heredoc state.
            if let Some(ref end_marker) = heredoc_end.clone() {
                if trimmed == end_marker.as_str() {
                    heredoc_end = None;
                }
                continue;
            }

            // Detect heredoc opener on the current line.
            // Look for `<<~WORD` or `<<-WORD` or `<<WORD` patterns.
            if let Some(marker) = detect_heredoc_marker(line) {
                heredoc_end = Some(marker);
            }

            // Skip comment lines — e.g. commented-out code `# )`.
            if trimmed.starts_with('#') {
                continue;
            }

            // Closing `)` on same line as last argument in multiline call
            if trimmed.ends_with(')') && !trimmed.starts_with(')') && trimmed.len() > 1 {
                // Skip if line starts with `}` — this is a `})` pattern where `}` closes
                // a hash argument and `)` closes the enclosing method call.  This is the
                // rubocop "symmetrical" style and is not flagged.
                if trimmed.starts_with('}') {
                    continue;
                }

                // Skip if trimmed starts with `\` — this is likely a regex or string
                // escape sequence (e.g. `\)` inside a multiline /regex/x), not Ruby code.
                if trimmed.starts_with('\\') {
                    continue;
                }

                // Skip if parentheses are balanced on this line — the trailing `)`
                // closes a `(` opened on the same line, not a previous-line multiline call.
                let open_count = trimmed.bytes().filter(|&b| b == b'(').count();
                let close_count = trimmed.bytes().filter(|&b| b == b')').count();
                if open_count >= close_count {
                    continue;
                }

                // Check if previous lines have unclosed `(`
                let mut j = i;
                let mut is_multiline = false;
                while j > 0 {
                    j -= 1;
                    let prev = lines[j].trim_end();
                    // Only treat as multiline-new-line style when the opening `(`
                    // is the last meaningful character on its line (bare paren).
                    // `foo(arg1,\n    arg2)` is valid symmetrical style — the `(`
                    // has content after it so it doesn't count here.
                    if prev.ends_with('(') {
                        is_multiline = true;
                        break;
                    }
                    if prev.trim().is_empty() { break; }
                }

                if is_multiline {
                    let indent = line.len() - line.trim_start().len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Closing `)` of multiline method call should be on its own line.".into(),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

/// Extract the heredoc end marker from a line containing a heredoc opener.
/// Returns `Some("MARKER")` if a `<<~MARKER`, `<<-MARKER`, or `<<MARKER` is found.
fn detect_heredoc_marker(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i + 1 < len {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut pos = i + 2;
            // skip optional `-` or `~`
            if pos < len && (bytes[pos] == b'-' || bytes[pos] == b'~') {
                pos += 1;
            }
            // skip optional quote character
            let quoted = pos < len && (bytes[pos] == b'\'' || bytes[pos] == b'"' || bytes[pos] == b'`');
            if quoted {
                pos += 1;
            }
            // Read the marker name: alphanumeric + underscore
            let start = pos;
            while pos < len && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }
            if pos > start {
                // close the quote if needed
                if quoted && pos < len {
                    pos += 1; // skip closing quote
                }
                let marker = &line[start..pos - if quoted { 1 } else { 0 }];
                let marker = marker.trim_matches(|c| c == '\'' || c == '"' || c == '`');
                if !marker.is_empty() {
                    return Some(marker.to_string());
                }
            }
        }
        i += 1;
    }
    None
}

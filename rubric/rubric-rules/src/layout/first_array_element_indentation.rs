use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FirstArrayElementIndentation;

fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i + 1 < len {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') { j += 1; }
            if j < len && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`') { j += 1; }
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            if j > start { return Some(line[start..j].to_string()); }
        }
        i += 1;
    }
    None
}

impl Rule for FirstArrayElementIndentation {
    fn name(&self) -> &'static str {
        "Layout/FirstArrayElementIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut in_heredoc: Option<String> = None;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();

            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // Detect heredoc opener — body starts on next line
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: opener line may contain a real `[`
            }

            if trimmed.starts_with('#') {
                continue;
            }

            // Detect line ending with `[` (multiline array start)
            if trimmed.ends_with('[') && i + 1 < n {
                let line_indent = line.len() - trimmed.len();
                // Consistent style: line_indent + 2
                let expected_consistent = line_indent + 2;
                // Special-inside-parentheses style: align to bracket column + 1 or + 2
                // bracket column = line_indent + position of `[` within trimmed
                let bracket_col = line_indent + trimmed.len() - 1;
                let expected_aligned_1 = bracket_col + 1;
                let expected_aligned_2 = bracket_col + 2;

                let next_line = &lines[i + 1];
                let actual_indent = next_line.len() - next_line.trim_start().len();

                if !next_line.trim().is_empty()
                    && actual_indent != expected_consistent
                    && actual_indent != expected_aligned_1
                    && actual_indent != expected_aligned_2
                {
                    let line_start = ctx.line_start_offsets[i + 1] as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "First array element should be indented {} spaces, not {}.",
                            expected_consistent, actual_indent
                        ),
                        range: TextRange::new(line_start, line_start + actual_indent as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

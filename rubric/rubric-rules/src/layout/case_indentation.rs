use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CaseIndentation;

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

impl Rule for CaseIndentation {
    fn name(&self) -> &'static str {
        "Layout/CaseIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track current `case` indentation
        // Stack of case indentations
        let mut case_stack: Vec<usize> = Vec::new();
        // General depth tracking for other openers
        let mut depth_stack: Vec<bool> = Vec::new(); // true = case opener
        // Heredoc tracking — skip body lines
        let mut in_heredoc: Option<String> = None;

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                i += 1;
                continue;
            }

            // Detect heredoc opener — body starts on next line
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: opener line contains real Ruby
            }

            if trimmed.starts_with("case ") || trimmed == "case" {
                case_stack.push(indent);
                depth_stack.push(true);
            } else if trimmed.starts_with("def ") || trimmed == "def"
                || trimmed.starts_with("class ") || trimmed.starts_with("module ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed.starts_with("begin")
                || trimmed == "do" || trimmed.ends_with(" do") {
                depth_stack.push(false);
            } else if trimmed == "end" || trimmed.starts_with("end ") {
                if let Some(is_case) = depth_stack.pop() {
                    if is_case {
                        case_stack.pop();
                    }
                }
            } else if trimmed.starts_with("when ") || trimmed == "when" {
                // `when` should be at the same indentation as `case`
                if let Some(&case_indent) = case_stack.last() {
                    if indent != case_indent {
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`when` indentation ({}) does not match `case` indentation ({}).",
                                indent, case_indent
                            ),
                            range: TextRange::new(
                                line_start + indent as u32,
                                line_start + indent as u32 + 4,
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            i += 1;
        }

        diags
    }
}

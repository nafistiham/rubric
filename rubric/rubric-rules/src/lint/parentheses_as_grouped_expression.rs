use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct ParenthesesAsGroupedExpression;

/// All Ruby keywords that may legitimately precede a `(` -- rubocop never flags
/// these because they are control-flow / declaration keywords, not method calls.
const KEYWORDS: &[&str] = &[
    "if", "unless", "while", "until", "return", "and", "or", "not", "do",
    "end", "def", "class", "module", "elsif", "else", "when", "rescue",
    "ensure", "yield", "super", "case", "for", "begin", "then", "in",
    "defined",
];

/// When the first argument of a method call is a grouped expression followed by
/// a comma -- e.g. `link_to (expr || other), path` -- rubocop does NOT flag it,
/// because `(expr)` there is unambiguously a grouped expression serving as one
/// of multiple arguments, not potentially the method's own parenthesised arg
/// list.  Detect this by finding the closing `)` at balanced depth and checking
/// that the next non-space character is a `,`.
fn grouped_expr_before_comma(line: &[u8], open_paren: usize) -> bool {
    let n = line.len();
    let mut depth: i32 = 0;
    let mut i = open_paren;
    while i < n {
        match line[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let mut j = i + 1;
                    while j < n && line[j] == b' ' {
                        j += 1;
                    }
                    return j < n && line[j] == b',';
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Extract the heredoc terminator word from a line containing `<<~TERM`,
/// `<<-TERM`, or `<<TERM` (optionally quoted).  Returns `None` when no
/// heredoc opener is present.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
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
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') {
                j += 1;
            }
            if j < len && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`') {
                j += 1;
            }
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }
        i += 1;
    }
    None
}

impl Rule for ParenthesesAsGroupedExpression {
    fn name(&self) -> &'static str {
        "Lint/ParenthesesAsGroupedExpression"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc: Option<String> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Heredoc end check
            if let Some(ref terminator) = in_heredoc.clone() {
                if line.trim() == terminator.as_str() {
                    in_heredoc = None;
                }
                continue; // skip heredoc body lines entirely
            }

            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect heredoc openers but do not skip this line itself;
            // the code before the <<TERM opener is still valid Ruby
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
            }

            let bytes = trimmed.as_bytes();
            let n = bytes.len();
            let mut pos = 0;

            // Read the leading identifier (potential method name)
            while pos < n && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }

            // We need at least one identifier character followed by ` (`
            if pos == 0 || pos >= n || bytes[pos] != b' ' {
                continue;
            }

            // Skip any extra spaces after the identifier
            let mut j = pos + 1;
            while j < n && bytes[j] == b' ' {
                j += 1;
            }

            if j >= n || bytes[j] != b'(' {
                continue;
            }

            let method_name = std::str::from_utf8(&bytes[..pos]).unwrap_or("");

            // Skip Ruby keywords -- they legitimately appear before `(`
            if KEYWORDS.contains(&method_name) {
                continue;
            }

            // Skip `method (grouped_expr), more_args` -- the `(...)` is a
            // grouped expression as one argument among several; rubocop does
            // not flag this pattern.
            if grouped_expr_before_comma(bytes, j) {
                continue;
            }

            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let space_pos = (line_start + indent + pos) as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Avoid space between method name and `(`; it looks like a grouped expression.".into(),
                range: TextRange::new(space_pos, space_pos + 1),
                severity: Severity::Warning,
            });
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: String::new(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

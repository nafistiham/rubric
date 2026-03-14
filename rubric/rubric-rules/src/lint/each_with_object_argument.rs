use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EachWithObjectArgument;

/// Returns the index of the comment character `#` on the line, ignoring
/// `#` that appear inside string literals.
/// Returns `None` if no real comment exists.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        i += 1;
    }
    None
}

/// Returns true if the argument immediately following the opening `(` of
/// `each_with_object(` is an immutable literal (integer, symbol, true/false/nil).
fn is_immutable_argument(after_paren: &str) -> bool {
    let s = after_paren.trim_start();

    // Integer literal: one or more digits
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        return true;
    }

    // Symbol literal: `:word`
    if s.starts_with(':') {
        let rest = &s[1..];
        if rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
            return true;
        }
    }

    // true, false, nil — must be followed by a word boundary
    for keyword in &["true", "false", "nil"] {
        if s.starts_with(keyword) {
            let next = s.as_bytes().get(keyword.len()).copied();
            let at_boundary = next.map_or(true, |b| !b.is_ascii_alphanumeric() && b != b'_');
            if at_boundary {
                return true;
            }
        }
    }

    false
}

impl Rule for EachWithObjectArgument {
    fn name(&self) -> &'static str {
        "Lint/EachWithObjectArgument"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let pattern = ".each_with_object(";

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Only scan up to the real comment character
            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];

            let line_start = ctx.line_start_offsets[i] as usize;
            let mut search = 0usize;

            while search < scan_slice.len() {
                if let Some(rel) = scan_slice[search..].find(pattern) {
                    let abs = search + rel;
                    let after_paren = &scan_slice[abs + pattern.len()..];

                    if is_immutable_argument(after_paren) {
                        let start = (line_start + abs) as u32;
                        let end = (line_start + abs + pattern.len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "The argument to each_with_object should not be immutable.".into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                    }

                    search = abs + pattern.len();
                } else {
                    break;
                }
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FileNull;

const MESSAGE: &str = "Use File::NULL instead of '/dev/null'.";

/// Returns the byte offset of a comment `#` on the line, ignoring `#` inside
/// string literals or interpolations. Returns `None` if no real comment.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut interp_depth: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(b'"') if bytes[i..].starts_with(b"#{") => {
                interp_depth += 1;
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d && interp_depth == 0 => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        if interp_depth > 0 && bytes[i] == b'}' {
            interp_depth -= 1;
        }
        i += 1;
    }
    None
}

impl Rule for FileNull {
    fn name(&self) -> &'static str {
        "Style/FileNull"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Quick pre-screen: must contain /dev/null
            if !line.contains("/dev/null") {
                continue;
            }

            let scan_end = comment_start(line).unwrap_or(line.len());
            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Find all occurrences (double or single quoted) within the code portion
            let mut search = 0usize;
            while search < scan_end {
                let remaining = &bytes[search..scan_end];
                let needle_dq = b"\"/dev/null\"";
                let needle_sq = b"'/dev/null'";

                let found = remaining
                    .windows(needle_dq.len())
                    .enumerate()
                    .find(|(_, w)| *w == needle_dq || *w == needle_sq);

                match found {
                    None => break,
                    Some((rel, _)) => {
                        let abs = search + rel;
                        let start = (line_start + abs) as u32;
                        // The match length is the full quoted literal (11 bytes)
                        let end = start + needle_dq.len() as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: MESSAGE.into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                        search = abs + needle_dq.len();
                    }
                }
            }
        }

        diags
    }
}

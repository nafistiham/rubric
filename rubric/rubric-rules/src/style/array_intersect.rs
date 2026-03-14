use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ArrayIntersect;

impl Rule for ArrayIntersect {
    fn name(&self) -> &'static str {
        "Style/ArrayIntersect"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Find comment boundary so we don't scan into comments
            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];

            // Detect `).any?` and ` & ` (set intersection) on the same line
            if scan_slice.contains(").any?") && scan_slice.contains(" & ") {
                let line_start = ctx.line_start_offsets[i] as usize;
                // Point to the start of the line for the diagnostic range
                let start = line_start as u32;
                let end = start + line.len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use Array#intersect? instead of (array1 & array2).any?.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Returns the index of the comment character `#` on the line, ignoring
/// `#` that appear inside string literals.
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

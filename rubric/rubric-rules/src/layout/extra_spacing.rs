use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ExtraSpacing;

impl Rule for ExtraSpacing {
    fn name(&self) -> &'static str {
        "Layout/ExtraSpacing"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            // Skip indentation (leading whitespace)
            let indent_len = line.len() - line.trim_start().len();
            let content = &line[indent_len..];

            // Skip pure comment lines
            let content_trimmed = content.trim_start();
            if content_trimmed.starts_with('#') {
                continue;
            }

            // Scan for consecutive spaces outside strings
            let bytes = content.as_bytes();
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
                    None if b == b'#' => break, // inline comment
                    None => {}
                }
                // Check for two or more consecutive spaces
                if b == b' ' && j + 1 < len && bytes[j + 1] == b' ' {
                    let span_start = j;
                    while j < len && bytes[j] == b' ' {
                        j += 1;
                    }
                    let span_end = j;
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let abs_start = (line_start + indent_len + span_start) as u32;
                    let abs_end = (line_start + indent_len + span_end) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra spacing detected.".into(),
                        range: TextRange::new(abs_start, abs_end),
                        severity: Severity::Warning,
                    });
                    continue;
                }
                j += 1;
            }
        }

        diags
    }
}

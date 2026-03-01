use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HeredocIndentation;

impl Rule for HeredocIndentation {
    fn name(&self) -> &'static str {
        "Layout/HeredocIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let line = &lines[i];
            // Look for `<<~IDENTIFIER`
            if let Some(marker_pos) = line.find("<<~") {
                let marker_start = marker_pos + 3;
                let marker = line[marker_start..].trim();
                // Get the heredoc identifier (until whitespace or end of line)
                let marker_end = marker.find(|c: char| c.is_whitespace()).unwrap_or(marker.len());
                let heredoc_id = &marker[..marker_end];

                if heredoc_id.is_empty() {
                    i += 1;
                    continue;
                }

                // Read heredoc content lines until we hit the closing marker
                i += 1;
                while i < n {
                    let content_line = &lines[i];
                    let content_trimmed = content_line.trim();

                    // Check if this is the closing marker
                    if content_trimmed == heredoc_id {
                        break;
                    }

                    // Check indentation: for `<<~`, content should have at least some indentation
                    if !content_line.trim_end().is_empty() {
                        let has_indent = content_line.starts_with(' ') || content_line.starts_with('\t');
                        if !has_indent {
                            let line_start = ctx.line_start_offsets[i] as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Heredoc content should be indented for `<<~`.".into(),
                                range: TextRange::new(line_start, line_start + content_line.len() as u32),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    i += 1;
                }
            }
            i += 1;
        }

        diags
    }
}

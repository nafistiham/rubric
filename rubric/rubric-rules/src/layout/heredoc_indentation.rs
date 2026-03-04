use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HeredocIndentation;

/// Extract the heredoc identifier from the text after `<<~` or `<<-`.
///
/// Ruby's heredoc syntax allows several forms:
///   <<~IDENTIFIER          — bare identifier
///   <<~IDENTIFIER.method   — chained method call; terminator is just IDENTIFIER
///   <<~IDENTIFIER)         — heredoc inside expression; terminator is IDENTIFIER
///   <<~'IDENTIFIER'        — single-quoted (no interpolation); terminator is IDENTIFIER
///   <<~"IDENTIFIER"        — double-quoted; terminator is IDENTIFIER
///   <<~`IDENTIFIER`        — backtick; terminator is IDENTIFIER
///
/// This function returns the bare identifier used as the terminator line.
fn extract_heredoc_id(after_marker: &str) -> Option<&str> {
    let s = after_marker.trim_start();
    if s.is_empty() {
        return None;
    }

    // Strip surrounding quotes if present
    let s = if s.starts_with('\'') || s.starts_with('"') || s.starts_with('`') {
        let quote = &s[..1];
        // find closing quote
        let rest = &s[1..];
        let end = rest.find(quote)?;
        &rest[..end]
    } else {
        // Bare identifier: read until first non-word character (letters, digits, underscore)
        // The terminator is the longest run of word chars at the start.
        let end = s
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(s.len());
        &s[..end]
    };

    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

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
                let after = &line[marker_pos + 3..];
                if let Some(heredoc_id) = extract_heredoc_id(after) {
                    // Read heredoc content lines until we hit the closing marker.
                    // The closing marker for `<<~` is the identifier alone on a line
                    // (possibly with leading whitespace which squiggly heredocs strip).
                    i += 1;
                    while i < n {
                        let content_line = &lines[i];
                        let content_trimmed = content_line.trim();

                        // Check if this is the closing marker
                        if content_trimmed == heredoc_id {
                            break;
                        }

                        // For `<<~` heredocs: the body lines are EXPECTED to be indented
                        // in source (Ruby strips the minimum leading whitespace at runtime).
                        // We only flag lines that have NO indentation at all AND are non-empty,
                        // which is a genuine violation (content flush against column 1 inside
                        // a squiggly heredoc is invalid style).
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
            }
            i += 1;
        }

        diags
    }
}

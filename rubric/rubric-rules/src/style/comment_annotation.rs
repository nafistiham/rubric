use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CommentAnnotation;

/// Annotation keywords that rubocop recognises.
const KEYWORDS: &[&str] = &[
    "TODO", "FIXME", "OPTIMIZE", "HACK", "REVIEW", "NOTE", "XXX",
];

/// Returns the annotation keyword found in `comment_text` (the text after `#`),
/// together with its byte offset within `comment_text`, or `None` if no keyword
/// is present.
///
/// A keyword must be surrounded by non-word characters (or at a boundary) to
/// avoid matching substrings like "TODOS".
fn find_annotation_keyword(comment_text: &str) -> Option<(&'static str, usize)> {
    for &kw in KEYWORDS {
        if let Some(pos) = comment_text.find(kw) {
            // Word-boundary check: char before must be non-word (or start of text)
            let before_ok = pos == 0 || {
                let ch = comment_text[..pos].chars().next_back().unwrap();
                !ch.is_alphanumeric() && ch != '_'
            };
            // Word-boundary check: char after must be non-word (or end of text)
            let after_pos = pos + kw.len();
            let after_ok = after_pos >= comment_text.len() || {
                let ch = comment_text[after_pos..].chars().next().unwrap();
                !ch.is_alphanumeric() && ch != '_'
            };

            if before_ok && after_ok {
                return Some((kw, pos));
            }
        }
    }
    None
}

/// Returns `true` when the annotation is correctly formatted.
///
/// Correct format: `KEYWORD: <non-empty description>`
/// i.e. after the keyword there is `: ` (colon + space) followed by at least
/// one non-whitespace character.
fn is_well_formed(comment_text: &str, kw: &str, kw_pos: usize) -> bool {
    let after_kw = &comment_text[kw_pos + kw.len()..];
    // Must start with ": " (colon then at least one space)
    if !after_kw.starts_with(": ") {
        return false;
    }
    // The description after ": " must not be empty / only whitespace
    let description = after_kw[2..].trim();
    !description.is_empty()
}

impl Rule for CommentAnnotation {
    fn name(&self) -> &'static str {
        "Style/CommentAnnotation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Must be a comment line
            if !trimmed.starts_with('#') {
                continue;
            }

            // Strip the leading `#` to get the comment text
            let comment_text = &trimmed[1..];

            if let Some((kw, kw_pos)) = find_annotation_keyword(comment_text) {
                if !is_well_formed(comment_text, kw, kw_pos) {
                    let line_start = ctx.line_start_offsets[i] as u32;
                    // Point the diagnostic at the whole line
                    let line_end = line_start + line.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Annotation comment, `{}`, is missing a note.",
                            kw
                        ),
                        range: TextRange::new(line_start, line_end),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

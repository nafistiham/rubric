use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLineAfterMagicComment;

fn is_magic_comment(line: &str) -> bool {
    let t = line.trim_start_matches('#').trim_start();
    t.starts_with("frozen_string_literal:")
        || t.starts_with("encoding:")
        || t.starts_with("coding:")
        || t.starts_with("warn_indent:")
        || t.starts_with("typed:")
}

impl Rule for EmptyLineAfterMagicComment {
    fn name(&self) -> &'static str {
        "Layout/EmptyLineAfterMagicComment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let lines = &ctx.lines;
        let n = lines.len();

        // Find the last consecutive magic comment at the top.
        let mut last_magic_idx: Option<usize> = None;
        for i in 0..n {
            let trimmed = lines[i].trim();
            if trimmed.starts_with('#') && is_magic_comment(trimmed) {
                last_magic_idx = Some(i);
            } else {
                break;
            }
        }

        let last_magic_idx = match last_magic_idx {
            None => return vec![],
            Some(idx) => idx,
        };

        // If the magic comment is the last line, no violation.
        let next_idx = last_magic_idx + 1;
        if next_idx >= n {
            return vec![];
        }

        // The line immediately after must be blank.
        let next_line = lines[next_idx].trim();
        if next_line.is_empty() {
            return vec![];
        }

        // Report on the line after the magic comment block.
        let line_start = ctx.line_start_offsets[next_idx];
        let line_end = line_start + lines[next_idx].len() as u32;
        vec![Diagnostic {
            rule: self.name(),
            message: "Add an empty line after the magic comments.".into(),
            range: TextRange::new(line_start, line_end),
            severity: Severity::Warning,
        }]
    }
}

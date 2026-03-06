use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AmbiguousOperator;

/// Ruby keywords that can legally be followed by `*arr` or `&blk` without
/// the `*`/`&` being ambiguous.  In these positions the operator is always
/// a splat or block-pass, never multiplication/binary-and.
const UNAMBIGUOUS_KEYWORDS: &[&str] = &[
    "rescue", "return", "yield", "raise", "fail",
    "next", "break", "throw", "and", "or", "not",
    "if", "unless", "while", "until", "when",
];

fn preceding_word(line: &[u8], space_pos: usize) -> &[u8] {
    if space_pos == 0 { return b""; }
    let mut end = space_pos;
    while end > 0 && (line[end - 1].is_ascii_alphanumeric() || line[end - 1] == b'_') {
        end -= 1;
    }
    &line[end..space_pos]
}

/// If `line` opens a heredoc, return the terminator string.
fn heredoc_terminator(line: &[u8]) -> Option<Vec<u8>> {
    let n = line.len();
    let mut i = 0;
    while i + 1 < n {
        if line[i] == b'<' && line[i + 1] == b'<' {
            i += 2;
            if i < n && (line[i] == b'-' || line[i] == b'~') { i += 1; }
            if i < n && (line[i] == b'\'' || line[i] == b'"' || line[i] == b'`') { i += 1; }
            let start = i;
            while i < n && (line[i].is_ascii_alphanumeric() || line[i] == b'_') { i += 1; }
            if i > start { return Some(line[start..i].to_vec()); }
        } else { i += 1; }
    }
    None
}

impl Rule for AmbiguousOperator {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousOperator"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let mut in_heredoc: Option<Vec<u8>> = None;

        for (i, line) in lines.iter().enumerate() {
            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                continue;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();

            // Detect heredoc openers (body starts on next line)
            if let Some(term) = heredoc_terminator(bytes) {
                in_heredoc = Some(term);
            }

            let len = bytes.len();
            let mut j = 0;
            let mut in_string: Option<u8> = None;

            while j < len {
                let b = bytes[j];

                // ── String state: skip operators inside string literals ─────
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' || b == b'`' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment
                    None => {}
                }

                // Look for ` *word` or ` &word` pattern (space before * or & then word char)
                if b == b' ' && j + 2 < len {
                    let next = bytes[j + 1];
                    if (next == b'*' || next == b'&') && j + 2 < len {
                        let after_op = bytes[j + 2];
                        if after_op.is_ascii_alphabetic() || after_op == b'_' {
                            // Check that we're in a method call context (prev char is word/paren)
                            let prev_ok = j > 0 && (bytes[j - 1].is_ascii_alphanumeric() || bytes[j - 1] == b'_' || bytes[j - 1] == b')');
                            if prev_ok {
                                // Skip if the preceding word is a Ruby keyword where
                                // `*`/`&` is unambiguously a splat/block-pass.
                                let word = preceding_word(bytes, j);
                                let is_keyword = UNAMBIGUOUS_KEYWORDS.iter()
                                    .any(|kw| kw.as_bytes() == word);
                                if !is_keyword {
                                    let line_start = ctx.line_start_offsets[i] as usize;
                                    let op_pos = (line_start + j + 1) as u32;
                                    diags.push(Diagnostic {
                                        rule: self.name(),
                                        message: format!(
                                            "Ambiguous `{}` operator. Use parentheses to clarify intent.",
                                            next as char
                                        ),
                                        range: TextRange::new(op_pos, op_pos + 1),
                                        severity: Severity::Warning,
                                    });
                                }
                            }
                        }
                    }
                }
                j += 1;
            }
        }

        diags
    }
}

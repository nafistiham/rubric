use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineMethodCallIndentation;

/// Returns the byte index of the first `#` that is not inside a string literal,
/// or `None` if no such `#` exists.
fn find_inline_comment_start(line: &str) -> Option<usize> {
    let mut in_double = false;
    let mut in_single = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut byte_offset = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Handle escape sequences inside strings
        if (in_double || in_single) && ch == '\\' && i + 1 < chars.len() {
            // Skip the escaped character
            byte_offset += ch.len_utf8();
            i += 1;
            byte_offset += chars[i].len_utf8();
            i += 1;
            continue;
        }

        if ch == '"' && !in_single {
            in_double = !in_double;
        } else if ch == '\'' && !in_double {
            in_single = !in_single;
        } else if ch == '#' && !in_double && !in_single {
            return Some(byte_offset);
        }

        byte_offset += ch.len_utf8();
        i += 1;
    }

    None
}

/// Extracts the heredoc terminator identifier from a line that opens a heredoc.
/// Handles `<<TERM`, `<<~TERM`, `<<-TERM`, `<<"TERM"`, `<<'TERM'`, `<<`TERM``,
/// and chained heredocs like `<<~TERM.freeze`.
/// Returns the bare terminator string (without quotes/tilde/dash) or None.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i + 1 < len {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            i += 2;
            // optional - or ~
            if i < len && (bytes[i] == b'-' || bytes[i] == b'~') {
                i += 1;
            }
            if i >= len {
                break;
            }
            // optional quote
            let quote = if bytes[i] == b'"' || bytes[i] == b'\'' || bytes[i] == b'`' {
                let q = bytes[i];
                i += 1;
                Some(q)
            } else {
                None
            };

            // collect identifier chars
            let start = i;
            while i < len {
                let b = bytes[i];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    i += 1;
                } else {
                    break;
                }
            }
            if i > start {
                let term = &line[start..i];
                if !term.is_empty() {
                    // validate closing quote if opened
                    if let Some(q) = quote {
                        if i < len && bytes[i] == q {
                            // fine
                        }
                    }
                    return Some(term.to_string());
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

/// Returns true if `line` opens a multi-line regex literal, i.e. the line
/// contains an unmatched `/` that starts a regex (not a division operator).
/// This is a heuristic: a `/` after `=`, `(`, `,`, `[`, `{`, or at the start
/// of a statement is a regex opener.
fn opens_multiline_regex(line: &str) -> bool {
    let trimmed = line.trim_end();
    // Must end with / optionally followed by flags letters (ioxm etc.)
    // to be a multiline regex opener — but actually in Ruby multiline regex
    // opened with /\n... the opening line ends with just `/` (with optional space).
    // A simpler heuristic: the trimmed code part ends with `/ ` or just `/`
    // after stripping inline comments, AND the preceding context looks like an
    // assignment or method call argument.
    // We focus on the most common pattern: `= /` at end of code part.
    let code_part = match find_inline_comment_start(trimmed) {
        Some(idx) => trimmed[..idx].trim_end(),
        None => trimmed,
    };
    if code_part.ends_with('/') {
        // Check there's a prior `=` or `(` suggesting regex context
        let before = code_part[..code_part.len() - 1].trim_end();
        return before.ends_with('=')
            || before.ends_with('(')
            || before.ends_with(',')
            || before.ends_with('[')
            || before.ends_with('{')
            || before.is_empty();
    }
    false
}

impl Rule for MultilineMethodCallIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodCallIndentation"
    }

    /// Disabled by default: the current implementation only detects trailing
    /// dots (the line-continuation marker) but does not verify whether the
    /// following line's indentation is correct. This produces false positives
    /// on all well-formatted trailing-dot style chains. Re-enable only after
    /// the indentation-check logic is implemented.
    fn default_enabled(&self) -> bool {
        false
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // State for heredoc body tracking
        // Stack of heredoc terminators we are waiting to see (one per nested heredoc).
        let mut heredoc_terminators: Vec<String> = Vec::new();

        // State for multi-line regex literal tracking
        let mut in_multiline_regex = false;

        // State for trailing-dot chain indentation tracking
        let mut in_trailing_dot_chain = false;
        let mut chain_base_indent: usize = 0;

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_end();

            // ── Heredoc terminator check ────────────────────────────────────
            // A terminator line is one that matches the expected terminator exactly
            // (possibly with leading spaces for squiggly heredocs).
            if !heredoc_terminators.is_empty() {
                let stripped = trimmed.trim_start();
                // Check innermost terminator first
                if let Some(term) = heredoc_terminators.last() {
                    if stripped == term.as_str() {
                        heredoc_terminators.pop();
                        // Do not flag the terminator line itself; continue to next line
                        // but we need to collect any NEW heredoc openers on this line
                        // (terminator lines can't have code after them in Ruby).
                        continue;
                    }
                }
                // We are inside a heredoc body — but we still need to look for
                // nested heredoc openers on this line (not possible in Ruby mid-body,
                // so just skip the line entirely).
                continue;
            }

            // ── Multi-line regex body check ─────────────────────────────────
            if in_multiline_regex {
                // The closing `/` of a multi-line regex appears on its own line
                // (possibly with flags: `/iox`). Heuristic: a line whose non-space
                // content starts with `/` (optionally followed by flag chars) closes it.
                let stripped = trimmed.trim_start();
                if stripped.starts_with('/') {
                    let after_slash = stripped[1..].trim_start();
                    // flags are only ascii letters
                    let all_flags = after_slash.chars().all(|c| c.is_ascii_alphabetic());
                    if all_flags {
                        in_multiline_regex = false;
                        continue;
                    }
                }
                // Inside the regex body — skip
                continue;
            }

            // ── Skip pure comment lines ─────────────────────────────────────
            if trimmed.trim_start().starts_with('#') {
                continue;
            }

            // ── Strip inline comment ────────────────────────────────────────
            let code_part = match find_inline_comment_start(trimmed) {
                Some(idx) => trimmed[..idx].trim_end(),
                None => trimmed,
            };

            // ── Collect heredoc openers on this line ────────────────────────
            // A line may open zero, one, or more heredocs.  We must push all
            // terminators we find (they close in FIFO order after the current line).
            {
                let mut scan = code_part;
                loop {
                    match extract_heredoc_terminator(scan) {
                        Some(term) => {
                            // Find where `<<` starts in scan and advance past it
                            if let Some(pos) = scan.find("<<") {
                                // push terminator to be matched later
                                heredoc_terminators.push(term);
                                scan = &scan[pos + 2..];
                            } else {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }

            // ── Check for multi-line regex opener ──────────────────────────
            if opens_multiline_regex(code_part) {
                in_multiline_regex = true;
                // The opening line itself ends with `/` — not a trailing dot — so
                // no diagnostic for it. Move to next line.
                continue;
            }

            // ── Trailing-dot chain indentation check ───────────────────────
            //
            // RuboCop's `indented` style: when a method chain uses trailing dots
            // (`.` at end of line), all continuation lines must be indented by
            // exactly `indentation_width` (2) spaces MORE than the chain's opener.
            //
            // We track:
            //   `chain_base_indent`: indentation of the first line in a chain
            //   `in_trailing_dot_chain`: whether we are inside such a chain
            //
            // When a continuation line's indentation doesn't match
            // `chain_base_indent + 2`, flag the continuation line.
            let current_indent = line.len() - line.trim_start().len();

            if in_trailing_dot_chain {
                let expected = chain_base_indent + 2;
                if current_indent != expected && !code_part.trim().is_empty() {
                    let line_start = ctx.line_start_offsets[i] as u32;
                    let indent_end = line_start + current_indent as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Use {} (not {}) spaces for method chain indentation.",
                            expected, current_indent
                        ),
                        range: TextRange::new(line_start, indent_end.max(line_start + 1)),
                        severity: Severity::Warning,
                    });
                }
                if !code_part.ends_with('.') {
                    in_trailing_dot_chain = false;
                }
            } else if code_part.ends_with('.') {
                in_trailing_dot_chain = true;
                chain_base_indent = current_indent;
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct Semicolon;

/// Scan through a line byte-by-byte tracking string, comment, and regex state.
/// Returns the position of the first `;` found outside a string, regex, or comment,
/// or `None` if no such semicolon exists.
/// Returns true if `bytes[i..]` starts with `keyword` followed by a non-word character
/// (or end of slice), and the byte before `i` is non-word (or `i == 0`).
fn at_keyword(bytes: &[u8], i: usize, keyword: &[u8]) -> bool {
    if !bytes[i..].starts_with(keyword) {
        return false;
    }
    // Check what follows the keyword
    let after = bytes.get(i + keyword.len()).copied().unwrap_or(0);
    if after.is_ascii_alphanumeric() || after == b'_' {
        return false;
    }
    // Check word boundary before
    if i > 0 {
        let prev = bytes[i - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return false;
        }
    }
    true
}

fn first_semicolon_outside_string_comment(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_delimited: Option<u8> = None; // inside "...", '...', or any same-char percent literal
    let mut in_percent_brace = false;          // inside %r{...} or %(... or %Q(... (brace-counted)
    let mut brace_open: u8 = b'{';
    let mut brace_close: u8 = b'}';
    let mut brace_depth: i32 = 0;
    // Track inline block depth: `def`, `class`, `module`, `begin` open blocks;
    // `end` closes them. A `;` at depth > 0 is a block-body separator, not a
    // statement separator, so it should not be flagged.
    let mut inline_block_depth: i32 = 0;
    let mut i = 0;

    while i < bytes.len() {
        // Inside %r{...} / %(...) / %Q(...) etc. with paired delimiters
        if in_percent_brace {
            match bytes[i] {
                b'\\' => { i += 2; continue; }
                b if b == brace_open => { brace_depth += 1; }
                b if b == brace_close => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        in_percent_brace = false;
                    }
                }
                _ => {}
            }
            i += 1;
            continue;
        }

        // Inside "...", '...', /regex/, or a same-char percent literal like %r!...!
        if let Some(delim) = in_delimited {
            match bytes[i] {
                b'\\' => { i += 2; continue; }
                b if b == delim => { in_delimited = None; }
                _ => {}
            }
            i += 1;
            continue;
        }

        // Not inside any literal
        match bytes[i] {
            b'\\' => { i += 2; continue; }
            b'"' | b'\'' => { in_delimited = Some(bytes[i]); }
            // %r, %(, %Q, %q, %w, %W, %i, %I, %x, %s literals
            b'%' if i + 1 < bytes.len() => {
                let next = bytes[i + 1];
                // Determine where the delimiter character is
                let (open_char, advance) =
                    if matches!(next, b'r' | b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b'x' | b's') {
                        (bytes.get(i + 2).copied().unwrap_or(0), 3usize)
                    } else if matches!(next, b'(' | b'[' | b'{' | b'<' | b'!' | b'|' | b'/' | b'@' | b'`') {
                        (next, 2usize)
                    } else {
                        (0, 1usize)
                    };

                if open_char != 0 {
                    let close_char = match open_char {
                        b'(' => b')',
                        b'[' => b']',
                        b'{' => b'}',
                        b'<' => b'>',
                        other => other,
                    };
                    if close_char != open_char {
                        // Paired delimiter — track depth
                        in_percent_brace = true;
                        brace_open = open_char;
                        brace_close = close_char;
                        brace_depth = 1;
                    } else {
                        // Same-char delimiter (e.g. %r!...!)
                        in_delimited = Some(close_char);
                    }
                    i += advance;
                    continue;
                }
                i += 1;
            }
            // /regex/ — only if preceded by an operator, delimiter, or method name
            b'/' => {
                // `/=` is divide-assign, not a regex — skip the `=` too
                if bytes.get(i + 1).copied() == Some(b'=') {
                    i += 1; // skip `=`; outer `i += 1` handles `/`
                } else {
                    let prev = bytes[..i].iter().rposition(|&b| b != b' ' && b != b'\t')
                        .map(|p| bytes[p]);
                    let is_regex_ctx = matches!(prev, None
                        | Some(b'[') | Some(b'(') | Some(b',') | Some(b'=')
                        | Some(b'!') | Some(b'|') | Some(b'&') | Some(b'?') | Some(b':'))
                        // Also treat `/` after a method name (alphabetic/`_`) as regex context.
                        // e.g. `match /regex/`, `gsub /pattern/`, `scan /re/`
                        || prev.map_or(false, |c| c.is_ascii_alphabetic() || c == b'_');
                    if is_regex_ctx {
                        in_delimited = Some(b'/');
                    }
                }
            }
            b'#' => return None, // comment
            b';' => {
                if inline_block_depth == 0 {
                    return Some(i);
                }
                // else: structural separator inside inline def/class/module — skip
            }
            // Track inline block depth for keywords that open a block body.
            // `def`, `class`, `class<<`, `module`, `begin` → depth++
            // `end` → depth-- (when followed by non-word char)
            b'd' if at_keyword(bytes, i, b"do") || at_keyword(bytes, i, b"def") => {
                inline_block_depth += 1;
            }
            b'c' if at_keyword(bytes, i, b"class") => {
                inline_block_depth += 1;
            }
            b'm' if at_keyword(bytes, i, b"module") => {
                inline_block_depth += 1;
            }
            b'b' if at_keyword(bytes, i, b"begin") => {
                inline_block_depth += 1;
            }
            b'e' if at_keyword(bytes, i, b"end") => {
                if inline_block_depth > 0 {
                    inline_block_depth -= 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Returns true if the `;` at `pos` is a trailing semicolon (only whitespace after it).
fn is_trailing_semicolon(line: &str, pos: usize) -> bool {
    line[pos + 1..].trim().is_empty()
}

/// Returns true if `line` opens a multiline regex (a regex-start `/` with no
/// matching closing `/` on the same line). Used to track cross-line state so
/// that `;` inside multiline regex character classes (`[!$()*+,;=]`) are not
/// flagged.
fn opens_multiline_regex(line: &str) -> bool {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    let mut in_string: Option<u8> = None; // inside '...' or "..."

    while i < n {
        let b = bytes[i];

        if let Some(delim) = in_string {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == delim {
                in_string = None;
            }
            i += 1;
            continue;
        }

        match b {
            b'#' => break, // comment — stop
            b'\'' | b'"' => {
                in_string = Some(b);
                i += 1;
            }
            b'/' => {
                // Check if this `/` is a regex opener
                let prev_non_space = (0..i)
                    .rev()
                    .find(|&j| bytes[j] != b' ' && bytes[j] != b'\t')
                    .map(|j| bytes[j]);
                let is_regex_start = match prev_non_space {
                    None => true,
                    Some(c) => matches!(c, b'=' | b'(' | b',' | b'[' | b'!' | b'&' | b'|'
                        | b'{' | b';' | b':' | b'<' | b'>' | b'+' | b'-' | b'*'
                        | b'%' | b'^' | b'~' | b'?' | b'\n'),
                };
                if is_regex_start {
                    // Scan forward for a closing unescaped `/`
                    let mut j = i + 1;
                    let mut found_close = false;
                    while j < n {
                        match bytes[j] {
                            b'\\' => { j += 2; continue; }
                            b'/' => { found_close = true; break; }
                            _ => {}
                        }
                        j += 1;
                    }
                    if !found_close {
                        return true;
                    }
                    // Skip past the closed regex
                    i = j + 1;
                    // Skip flags (i, m, x, etc.)
                    while i < n && bytes[i].is_ascii_alphabetic() {
                        i += 1;
                    }
                    continue;
                }
                i += 1;
            }
            _ => { i += 1; }
        }
    }
    false
}

impl Rule for Semicolon {
    fn name(&self) -> &'static str {
        "Style/Semicolon"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc: Option<String> = None;
        let mut in_multiline_regex = false;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines — `;` inside are non-Ruby content
            if let Some(ref term) = in_heredoc {
                if line.trim_end_matches(['\r', '\n']) == term.as_str()
                    || line.trim_end_matches(['\r', '\n']).trim_start() == term.as_str()
                {
                    in_heredoc = None;
                }
                continue;
            }

            // Skip multiline regex body lines. Exit when we hit the closing `/flags`.
            if in_multiline_regex {
                let t = line.trim();
                if t.starts_with('/') {
                    let after = t[1..].trim_start_matches(|c: char| c.is_ascii_alphabetic());
                    if after.is_empty() || after.starts_with('#') || after.trim().is_empty() {
                        in_multiline_regex = false;
                    }
                } else if t.starts_with(")/") || t.starts_with("}/" ) {
                    // )/iox or }/iox closing a grouped multiline regex
                    in_multiline_regex = false;
                }
                continue;
            }

            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Skip single-line method/class/module definitions and singleton class
            // expressions. Rubocop does not flag `;` in these structural forms:
            //   def foo; body; end
            //   class Foo; end / class Foo < Bar; end
            //   module Foo; end
            //   class << obj; self; end  (common singleton class idiom)
            // These are handled by Style/SingleLineMethods etc., not Style/Semicolon.
            let without_comment = trimmed.split_once(" #").map_or(trimmed, |(code, _)| code.trim_end());
            let ends_with_end = without_comment.ends_with(" end")
                || without_comment.ends_with(";end")
                || without_comment == "end";
            let is_single_line_structural = ends_with_end && (
                trimmed.starts_with("def ")
                    || trimmed.starts_with("def(")
                    || trimmed.starts_with("class ")
                    || trimmed.starts_with("class<<")
                    || trimmed.starts_with("(class ")
                    || trimmed.starts_with("(class<<")
                    || trimmed.starts_with("module ")
            );
            if is_single_line_structural {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;

            // Find the first `;` outside strings/comments
            if let Some(pos) = first_semicolon_outside_string_comment(line) {
                // Skip trailing semicolons (nothing substantive after them)
                if is_trailing_semicolon(line, pos) {
                    continue;
                }

                // Skip if nothing before the semicolon (just whitespace)
                if line[..pos].trim().is_empty() {
                    continue;
                }

                let start = (line_start + pos) as u32;
                let end = start + 1;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Do not use semicolons to terminate expressions.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }

            // Detect multiline regex opener on this line.
            if !in_multiline_regex && opens_multiline_regex(line) {
                in_multiline_regex = true;
            }

            // Detect heredoc opener on this line
            if in_heredoc.is_none() {
                let bytes = line.as_bytes();
                let mut search = bytes;
                while let Some(pos) = search.windows(2).position(|w| w == b"<<") {
                    let rest = &search[pos + 2..];
                    let rest = rest.strip_prefix(b"~").unwrap_or(rest);
                    let rest = rest.strip_prefix(b"-").unwrap_or(rest);
                    let rest = rest.strip_prefix(b"'").unwrap_or_else(|| rest.strip_prefix(b"\"").unwrap_or(rest));
                    let term_end = rest.iter().position(|&b| !b.is_ascii_alphanumeric() && b != b'_').unwrap_or(rest.len());
                    if term_end > 0 {
                        let term = std::str::from_utf8(&rest[..term_end]).unwrap_or("").to_string();
                        in_heredoc = Some(term);
                        break;
                    }
                    search = &search[pos + 2..];
                }
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_multiple_statements_on_one_line() {
        let src = "x = 1; y = 2\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for 'x = 1; y = 2', got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/Semicolon"));
    }

    #[test]
    fn detects_multiple_semicolons() {
        let src = "a = 1; b = 2; c = 3\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations for multiple semicolons"
        );
    }

    #[test]
    fn no_violation_for_clean_code() {
        let src = "x = 1\ny = 2\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_semicolon_in_string() {
        let src = "greeting = \"hello; world\"\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for semicolon in string, got: {:?}",
            diags
        );
    }

    #[test]
    fn no_violation_for_semicolon_in_comment() {
        let src = "x = 1 # this; is a comment\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for semicolon in comment, got: {:?}",
            diags
        );
    }

    #[test]
    fn no_violation_for_trailing_semicolon() {
        let src = "x = 1;\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for trailing semicolon, got: {:?}",
            diags
        );
    }

    #[test]
    fn uses_offending_fixture() {
        let offending = include_str!("../../tests/fixtures/style/semicolon/offending.rb");
        let ctx = LintContext::new(Path::new("test.rb"), offending);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations in offending.rb, got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/Semicolon"));
    }

    #[test]
    fn no_violation_on_passing_fixture() {
        let passing = include_str!("../../tests/fixtures/style/semicolon/passing.rb");
        let ctx = LintContext::new(Path::new("test.rb"), passing);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations in passing.rb, got: {:?}",
            diags
        );
    }
}

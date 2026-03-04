use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct BlockDelimiters;

impl Rule for BlockDelimiters {
    fn name(&self) -> &'static str {
        "Style/BlockDelimiters"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track open `{` that appear at end of a line (after a method call).
        // If the matching `}` is on a different line, flag the `{`.
        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed_start = line.trim_start();
            let trimmed = line.trim_end();

            // Skip comment lines — a line that starts with `#` after leading whitespace
            if trimmed_start.starts_with('#') {
                i += 1;
                continue;
            }

            // Check if line ends with `{` (block opener at end of line)
            if trimmed.ends_with('{') {
                // Find the `{` position
                let brace_pos = trimmed.rfind('{').expect("just confirmed it ends with {");
                let before_brace = &trimmed[..brace_pos].trim_end();
                // Only flag if there's a method call before the `{`
                // (i.e., line doesn't just start with `{`)
                if !before_brace.is_empty() {
                    let last_char = before_brace.chars().last();

                    // Skip if `{` is in a hash-literal or non-block context.
                    // Contexts where `{` is NOT a block:
                    //   - Hash assignment: last char is `=`, `:`, `,`, `(`, `[`, `{`
                    //   - Hash rocket RHS: last char is `>` (from `=>`)
                    //   - Shovel/append: last char is `<` (from `<<`)
                    //   - String interpolation: last char is `#` (from `#{`)
                    //   - Percent literal: last char is `%` or an alpha preceded by `%`
                    //     e.g. `%r{`, `%w{`, `%i{`, `%{`
                    //   - Escaped brace in regex/string: last char is `\` (from `\{`)
                    let is_non_block_context = matches!(
                        last_char,
                        Some('=') | Some(':') | Some(',') | Some('(') | Some('[') | Some('{')
                            | Some('>') // hash rocket `=>`
                            | Some('<') // shovel `<<`
                            | Some('#') // string interpolation `#{`
                            | Some('\\') // escaped brace `\{` in regex/string body
                    ) || is_percent_literal_opener(before_brace);

                    if is_non_block_context {
                        i += 1;
                        continue;
                    }

                    // Skip if this is a chained block: `}.method {` or `}.method.method {`
                    // The pattern is when `before_brace` contains a closing `}` followed by
                    // a method chain leading up to the `{`. This is the "braces for chaining"
                    // pattern that rubocop allows even in `line_count_based` mode.
                    if is_chained_block(before_brace) {
                        i += 1;
                        continue;
                    }

                    // Skip if this is a lambda body: `-> {` or `->(args) {`
                    // Scan backward from brace_pos through the bytes of the line.
                    let bytes = trimmed.as_bytes();
                    let is_lambda = {
                        // k points to last byte before `{` (we already stripped trailing ws via trimmed)
                        let mut k = brace_pos as isize - 1;
                        // skip spaces between `{` and what precedes it
                        while k >= 0 && bytes[k as usize] == b' ' { k -= 1; }
                        if k >= 0 && bytes[k as usize] == b')' {
                            // Possibly `->(args) {` — find matching `(`
                            let mut paren_depth = 1i32;
                            let mut m = k - 1;
                            while m >= 0 && paren_depth > 0 {
                                let c = bytes[m as usize];
                                if c == b')' { paren_depth += 1; }
                                else if c == b'(' { paren_depth -= 1; }
                                m -= 1;
                            }
                            // skip spaces before `(`
                            while m >= 0 && bytes[m as usize] == b' ' { m -= 1; }
                            // lambda if `->` immediately precedes the `(`
                            m >= 1
                                && bytes[m as usize] == b'>'
                                && bytes[(m - 1) as usize] == b'-'
                        } else if k >= 1
                            && bytes[k as usize] == b'>'
                            && bytes[(k - 1) as usize] == b'-'
                        {
                            // `-> {` — lambda with no args
                            true
                        } else {
                            false
                        }
                    };
                    if is_lambda {
                        i += 1;
                        continue;
                    }

                    // Skip if `lambda {` keyword form precedes the `{`
                    if is_lambda_keyword_before_brace(before_brace) {
                        i += 1;
                        continue;
                    }

                    // Now check if the matching `}` is on a different line
                    let mut depth = 1i32;
                    let mut j = i + 1;
                    while j < n && depth > 0 {
                        let next = &lines[j];
                        for ch in next.chars() {
                            if ch == '{' { depth += 1; }
                            else if ch == '}' { depth -= 1; }
                            if depth == 0 { break; }
                        }
                        if depth > 0 { j += 1; }
                    }
                    // If j != i, `}` is on a different line — multi-line brace block
                    if j > i {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let abs_pos = (line_start + brace_pos) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Multi-line block uses `{}` instead of `do..end`.".into(),
                            range: TextRange::new(abs_pos, abs_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            i += 1;
        }

        diags
    }
}

/// Returns true if `before_brace` ends with a percent literal opener like
/// `%r`, `%w`, `%i`, `%W`, `%I`, or bare `%`.
/// These open regex/string/array literals, not blocks.
fn is_percent_literal_opener(before_brace: &str) -> bool {
    let b = before_brace.as_bytes();
    let len = b.len();
    if len == 0 {
        return false;
    }
    let last = b[len - 1];
    if last == b'%' {
        return true;
    }
    // One alpha modifier letter preceded by `%`
    if last.is_ascii_alphabetic() && len >= 2 && b[len - 2] == b'%' {
        return true;
    }
    false
}

/// Returns true if the word `lambda` appears as the last token of `before_brace`.
fn is_lambda_keyword_before_brace(before_brace: &str) -> bool {
    let s = before_brace.trim_end();
    // Must end with `lambda` as a standalone word
    if !s.ends_with("lambda") {
        return false;
    }
    let prefix_len = s.len() - "lambda".len();
    // Word boundary before `lambda`
    if prefix_len == 0 {
        return true;
    }
    let prev = s.as_bytes()[prefix_len - 1];
    // Not alphanumeric or underscore — it's a word boundary
    !prev.is_ascii_alphanumeric() && prev != b'_'
}

/// Returns true if `before_brace` is a chained block pattern: the part before the
/// final `{` contains a closing `}` followed by a method chain (e.g. `expect { }.to change`).
///
/// This matches the rubocop "braces for chaining" pattern that rubocop allows even in
/// `line_count_based` mode — when a block's `}` is chained directly to the next call.
fn is_chained_block(before_brace: &str) -> bool {
    // Find the last `}` in before_brace. If there is one, and everything after it
    // looks like a method chain (`.method_name` or `.method_name(args)`), this is
    // a chained block and should not be flagged.
    let s = before_brace.trim_end();
    if let Some(close_brace_pos) = s.rfind('}') {
        // There is a `}` before the opening `{`. Check that what follows is a method chain.
        let after_close = &s[close_brace_pos + 1..].trim_start();
        // Method chain: starts with `.` followed by identifier chars
        if after_close.starts_with('.') {
            return true;
        }
    }
    false
}

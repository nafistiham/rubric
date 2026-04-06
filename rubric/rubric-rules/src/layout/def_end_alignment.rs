use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DefEndAlignment;

/// Extracts the heredoc terminator word from a line containing `<<`, `<<-`, or `<<~`.
/// Returns `None` if the line does not open a heredoc.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i + 1 < len {
        match in_str {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(d) if bytes[i] == d => { in_str = None; i += 1; continue; }
            Some(_) => { i += 1; continue; }
            None if bytes[i] == b'"' || bytes[i] == b'\'' => { in_str = Some(bytes[i]); i += 1; continue; }
            None if bytes[i] == b'#' => break,
            None => {}
        }
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') { j += 1; }
            let quote = if j < len && matches!(bytes[j], b'\'' | b'"' | b'`') {
                let q = bytes[j]; j += 1; Some(q)
            } else { None };
            let _ = quote;
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }
        i += 1;
    }
    None
}

/// Returns true if `trimmed` is an endless method (`def foo = expr` / `def foo(x) = expr`).
///
/// Key rule: the ` = ` that opens an endless method body must appear either
/// - right after the method name with no parentheses (and no commas at depth 0 before it),
///   e.g. `def foo = expr`
/// - after the closing `)` at depth 0, e.g. `def foo(x, y = 1) = expr`
///
/// A ` = ` that appears after a comma at depth 0 (or inside parens) is a default
/// parameter value, not the endless-method body separator.
/// A ` = ` that appears after a space-separated word (unparenthesized parameter)
/// is also a default parameter value, e.g. `def foo bar = default`.
fn is_endless_method(trimmed: &str) -> bool {
    let def_pos = match trimmed.find("def ") {
        Some(p) if p <= 20 => p,
        _ => return false,
    };
    let after_def = &trimmed[def_pos + 4..]; // skip "def "
    let bytes = after_def.as_bytes();
    let n = bytes.len();

    // Skip optional receiver (e.g. `self`, `opts`, `sw`) — just alphanumeric + `_`
    let mut i = 0;
    while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    // If followed by `.`, this was a receiver — skip the dot and scan the method name proper
    if i < n && bytes[i] == b'.' {
        i += 1;
        while i < n && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'_' | b'!' | b'?' | b'=' | b'[' | b']')) {
            i += 1;
        }
    } else {
        // No receiver — finish skipping remaining method-name chars (`!`, `?`, `=`, `[]`)
        while i < n && matches!(bytes[i], b'!' | b'?' | b'=' | b'[' | b']') {
            i += 1;
        }
    }

    if i >= n {
        return false;
    }

    match bytes[i] {
        b'(' => {
            // Parenthesized params: scan to the matching ')' then check for `=`
            let mut depth = 0i32;
            while i < n {
                match bytes[i] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 { i += 1; break; }
                    }
                    _ => {}
                }
                i += 1;
            }
            // Skip whitespace then check for `=` (not `==` or `=>`)
            while i < n && bytes[i] == b' ' { i += 1; }
            i < n && bytes[i] == b'='
                && (i + 1 >= n || (bytes[i + 1] != b'=' && bytes[i + 1] != b'>'))
        }
        b' ' => {
            // No parens — check if ` = ` immediately follows (endless `def foo = expr`)
            let rest = &after_def[i + 1..];
            rest.starts_with("= ")
                || rest.starts_with("=\t")
                || (rest.len() == 1 && rest == "=")
        }
        _ => false,
    }
}

/// If `bytes[i]` is `/` that starts a regex literal, returns the byte offset
/// just past the closing `/`. Otherwise returns `None`.
///
/// Heuristic: `/` starts a regex if the preceding non-whitespace character is
/// NOT a word character (`[a-zA-Z0-9_]`), `)`, `]`, or `}`.
fn skip_regex_literal(bytes: &[u8], i: usize) -> Option<usize> {
    if bytes.get(i).copied() != Some(b'/') { return None; }
    let prev = bytes[..i].iter().rev()
        .find(|&&c| c != b' ' && c != b'\t')
        .copied()
        .unwrap_or(0);
    if prev.is_ascii_alphanumeric() || prev == b'_'
        || prev == b')' || prev == b']' || prev == b'}' {
        return None;
    }
    let n = bytes.len();
    let mut j = i + 1;
    while j < n {
        if bytes[j] == b'\\' { j += 2; continue; }
        if bytes[j] == b'/' { return Some(j + 1); }
        j += 1;
    }
    Some(n) // unclosed regex — treat as extending to end of line
}

/// If `line` contains an unclosed percent literal (e.g. `%w(`, `%r{`, `%i[`),
/// returns `(opener_byte, closer_byte)`. The opener byte is the delimiter that
/// opens nesting (e.g. `{` for `%r{`), closer is the matching close delimiter.
/// Returns `None` if no unclosed percent literal is found.
fn find_unclosed_percent_literal(line: &str) -> Option<(u8, u8)> {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut in_string: Option<u8> = None;
    let mut i = 0;
    while i < n {
        match in_string {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(d) if bytes[i] == d => { in_string = None; }
            Some(_) => {}
            None if bytes[i] == b'\'' || bytes[i] == b'"' => { in_string = Some(bytes[i]); }
            None if bytes[i] == b'#' => {
                if i + 1 < n && bytes[i + 1] == b'{' { i += 2; continue; }
                break; // inline comment
            }
            None if bytes[i] == b'%' && i + 1 < n => {
                let next = bytes[i + 1];
                let (opener, closer, j_start) = if next == b'{' {
                    (b'{', b'}', i + 2)
                } else if matches!(next, b'r' | b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b'x' | b's')
                    && i + 2 < n
                {
                    let d = bytes[i + 2];
                    let close = match d { b'(' => b')', b'{' => b'}', b'[' => b']', b'<' => b'>', _ => 0 };
                    if close != 0 {
                        (d, close, i + 3)
                    } else if d.is_ascii_punctuation() && d != b'\\' && d != b'#' {
                        // Non-bracket delimiter (e.g. `|`, `!`, `/`): same char closes.
                        (d, d, i + 3)
                    } else {
                        (0, 0, 0)
                    }
                } else {
                    (0, 0, 0)
                };
                if closer != 0 {
                    // Scan to see if the closer appears on this same line.
                    let mut j = j_start;
                    let mut depth = 1usize;
                    while j < n && depth > 0 {
                        if bytes[j] == b'\\' { j += 2; continue; }
                        if opener != closer && bytes[j] == opener { depth += 1; }
                        else if bytes[j] == closer { depth -= 1; }
                        j += 1;
                    }
                    if depth > 0 {
                        // Didn't close on this line — multi-line percent literal.
                        return Some((opener, closer));
                    }
                    i = j;
                    continue;
                }
            }
            None => {}
        }
        i += 1;
    }
    None
}

/// Returns true if this line opens a multiline /regex/ that doesn't close on the
/// same line. Pattern: the code part ends with `/` after an assignment or opening-
/// paren context, e.g. `CONST = /`, `foo(`, `bar,` etc.
fn opens_unclosed_regex(line: &str) -> bool {
    let code_part = strip_trailing_comment(line.trim_end());
    if code_part.ends_with('/') {
        let before = code_part[..code_part.len() - 1].trim_end();
        return before.ends_with('=')
            || before.ends_with("=~")
            || before.ends_with("!~")
            || before.ends_with('(')
            || before.ends_with(',')
            || before.ends_with('[')
            || before.ends_with('{')
            || before.is_empty();
    }
    false
}

/// Strip a trailing `# comment` from a trimmed line.
/// Returns the line up to the comment `#`, trailing-whitespace-trimmed.
/// String interpolation `#{` is NOT treated as a comment.
fn strip_trailing_comment(t: &str) -> &str {
    match comment_start(t) {
        Some(pos) => t[..pos].trim_end_matches(|c: char| c == ' ' || c == '\t'),
        None => t,
    }
}

/// Returns true if `trimmed` is a one-liner like `class Foo; end`.
fn is_one_liner(trimmed: &str) -> bool {
    // Also check with any trailing comment stripped, so `def foo() end # doc`
    // is correctly recognised as a one-liner.
    for t in [
        trimmed.trim_end_matches(|c: char| c == ' ' || c == '\t'),
        strip_trailing_comment(trimmed),
    ] {
        if t.ends_with("; end") { return true; }
        // `def foo; body end` — body without trailing semicolon before end
        let is_opener = t.starts_with("def ")
            || t.starts_with("private def ") || t.starts_with("protected def ")
            || t.starts_with("private_class_method def ")
            || t.starts_with("module_function def ")
            || t.starts_with("class ") || t.starts_with("module ");
        if is_opener && (t.ends_with(" end") || t.ends_with(";end") || t.ends_with("\tend")) {
            // Guard: `def end` (a method whose name is `end`) is NOT a one-liner.
            // E.g. `def end\n  @end\nend` — the trailing `end` is the method name,
            // not the closing keyword. Detect this by checking if the content after
            // `def ` is exactly the word `end`.
            let is_def_end = t.find("def ").is_some_and(|p| &t[p + 4..] == "end");
            if !is_def_end {
                return true;
            }
        }
    }
    false
}

/// Returns true when a non-def block opener and its matching `end` appear on
/// the same line — e.g. `until cond; end`, `while x;end`, `foo do |x| end`.
/// These must NOT push an inner-construct frame because the scanner never sees
/// a standalone `end` line to pop it, leaving a phantom frame on the stack.
///
/// Lines starting with `rescue` or `ensure` are excluded: they don't open a
/// new block, they close one already opened earlier (e.g., `begin` on a prior
/// line), so they should act as closing tokens, not one-liner no-ops.
fn is_one_liner_block(t: &str) -> bool {
    // rescue/ensure lines with ; end are closers, not self-contained one-liners.
    if t.starts_with("rescue") || t.starts_with("ensure") { return false; }
    let t = t.trim_end_matches(|c: char| c == ' ' || c == '\t');
    t.ends_with("; end") || t.ends_with(";end")
        || t.ends_with(" end") || t.ends_with("\tend")
}

/// Finds the byte position of the first `#` that starts an inline comment,
/// properly skipping `#` inside string literals, `#{` interpolation, and
/// `/regex/` literals.
fn comment_start(t: &str) -> Option<usize> {
    let bytes = t.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    let mut in_string: Option<u8> = None; // current string delimiter (' or ")
    while i < n {
        match in_string {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(delim) if bytes[i] == delim => { in_string = None; }
            Some(_) => {}
            None => {
                if bytes[i] == b'\'' || bytes[i] == b'"' {
                    in_string = Some(bytes[i]);
                } else if bytes[i] == b'/' {
                    // Skip regex literals so that `#` inside `/regex/` is not
                    // mistaken for a comment start.
                    if let Some(end) = skip_regex_literal(bytes, i) {
                        i = end;
                        continue;
                    }
                } else if bytes[i] == b'#' {
                    // `#{` is string interpolation, not a comment
                    if i + 1 < n && bytes[i + 1] == b'{' { i += 2; continue; }
                    // `#` outside a string = comment start
                    return Some(i);
                }
            }
        }
        i += 1;
    }
    None
}

/// Returns true if the line ends with the `do` keyword (possibly without a preceding space),
/// e.g. `get('/*')do` or `items.each do`.
/// Requires the character before `do` to be non-alphanumeric/non-underscore so
/// words ending in `do` (like `redo`, `pseudo`) are not matched.
///
/// Inline comments are stripped (but `#{` interpolation is preserved), so
/// `# exceptions will do` at end does not trigger a false positive.
/// Since `comment_start` now properly skips `/regex/` literals, this function
/// correctly handles lines like `assert_raises /foo#bar/ do` (the `#` inside
/// the regex is not mistaken for a comment start).
fn ends_with_do_keyword(t: &str) -> bool {
    let stripped = match comment_start(t) {
        Some(pos) => t[..pos].trim_end_matches(|c: char| c == ' ' || c == '\t'),
        None => t.trim_end_matches(|c: char| c == ' ' || c == '\t'),
    };
    if stripped.ends_with("do") {
        let before_do = stripped.len().saturating_sub(2);
        if before_do == 0 { return true; }
        let c = stripped.as_bytes()[before_do - 1];
        if !(c.is_ascii_alphanumeric() || c == b'_') { return true; }
    }
    false
}

/// Returns true if the line contains ` do |params|` or ` do|params|` OUTSIDE
/// string literals, regex literals, and percent literals.
/// This replaces the naive `t.contains(" do |")` check which would incorrectly
/// match content inside strings (`"cascade do |t|"`) or regexes (`/each do |x|/`).
fn has_unquoted_do_with_params(t: &str) -> bool {
    let bytes = t.as_bytes();
    let n = bytes.len();
    let mut in_string: Option<u8> = None;
    let mut i = 0;
    while i < n {
        match in_string {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(d) if bytes[i] == d => { in_string = None; }
            Some(_) => {}
            None if bytes[i] == b'\'' || bytes[i] == b'"' => { in_string = Some(bytes[i]); }
            None if bytes[i] == b'/' => {
                // Skip regex literals so that ` do |` inside `/regex/` is not matched.
                if let Some(end) = skip_regex_literal(bytes, i) {
                    i = end;
                    continue;
                }
            }
            None if bytes[i] == b'%' && i + 1 < n => {
                // Skip single-line percent literals: %{...}, %q{...}, %w(...), etc.
                let next = bytes[i + 1];
                let (opener, closer, j_start) = if next == b'{' {
                    (b'{', b'}', i + 2)
                } else if matches!(next, b'r' | b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b'x' | b's')
                    && i + 2 < n
                {
                    let d = bytes[i + 2];
                    let close = match d { b'(' => b')', b'{' => b'}', b'[' => b']', b'<' => b'>', _ => 0 };
                    if close != 0 {
                        (d, close, i + 3)
                    } else if d.is_ascii_punctuation() && d != b'\\' && d != b'#' {
                        // Non-bracket delimiter (e.g. `|`, `!`, `/`): same char closes.
                        (d, d, i + 3)
                    } else {
                        (0, 0, 0)
                    }
                } else {
                    (0, 0, 0)
                };
                if closer != 0 {
                    let mut j = j_start;
                    let mut depth = 1usize;
                    while j < n && depth > 0 {
                        if bytes[j] == b'\\' { j += 2; continue; }
                        if opener != closer && bytes[j] == opener { depth += 1; }
                        else if bytes[j] == closer { depth -= 1; }
                        j += 1;
                    }
                    i = j;
                    continue;
                }
            }
            None if bytes[i] == b'#' => {
                if i + 1 < n && bytes[i + 1] == b'{' { i += 2; continue; } // interpolation
                break; // inline comment — stop scanning
            }
            None => {
                // Match ` do |` or ` do |` outside a string/regex/percent-literal
                if bytes[i] == b' ' && i + 2 < n
                    && bytes[i + 1] == b'd' && bytes[i + 2] == b'o'
                {
                    let next = bytes.get(i + 3).copied().unwrap_or(0);
                    if next == b'|' { return true; }
                    if next == b' ' && bytes.get(i + 4).copied().unwrap_or(0) == b'|' {
                        return true;
                    }
                }
            }
        }
        i += 1;
    }
    false
}

/// Returns true if the line contains `= KEYWORD ` or ends with `= KEYWORD`,
/// but only when the `=` is an assignment operator — NOT part of a method name
/// like `extra_params=` or `[]=`.
/// Exclusions:
///   - `=` preceded by `]` → `[]=` operator name
///   - `=` preceded by alphanumeric or `_` → setter method name (e.g. `method= if`)
fn eq_keyword(t: &str, keyword: &str) -> bool {
    let bytes = t.as_bytes();
    let n = bytes.len();
    let pat_space = format!("= {} ", keyword);
    let pat_end   = format!("= {}", keyword);
    for (i, _) in t.match_indices(&*pat_space) {
        if i == 0 { return true; }
        let prev = bytes[i - 1];
        // Skip if `=` is part of a method name (setter) or `[]=` operator
        if prev == b']' || prev.is_ascii_alphanumeric() || prev == b'_' { continue; }
        return true;
    }
    if t.ends_with(&*pat_end) {
        let i = n - pat_end.len();
        if i == 0 { return true; }
        let prev = bytes[i - 1];
        if prev == b']' || prev.is_ascii_alphanumeric() || prev == b'_' { return false; }
        return true;
    }
    false
}

/// Returns true if the line has ` do` immediately before an inline comment
/// or before a `;` (inline body), e.g. `foo do # :nodoc:`, `foo do; body`.
/// Both patterns open a block whose `end` is on a separate line.
fn has_do_before_comment(t: &str) -> bool {
    if let Some(pos) = t.find(" do ") {
        let after = t[pos + 4..].trim_start();
        if after.is_empty() || after.starts_with('#') || after.starts_with(';') {
            return true;
        }
    }
    // `foo do;body` — no space after `do`
    if let Some(pos) = t.find(" do;") {
        let before = if pos > 0 { t.as_bytes()[pos - 1] } else { 0 };
        if !(before.is_ascii_alphanumeric() || before == b'_') {
            return true;
        }
    }
    false
}

impl Rule for DefEndAlignment {
    fn name(&self) -> &'static str {
        "Layout/DefEndAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Unified stack: (line_idx, indent, is_def)
        // is_def=true = def/class/module (alignment checked), is_def=false = inner construct
        let mut stack: Vec<(usize, usize, bool)> = Vec::new();
        let mut in_heredoc: Option<String> = None;
        // Track multi-line /regex/ — body lines must not be parsed for keywords.
        let mut in_multiline_regex = false;
        // Track multi-line percent literals (`%r{...}`, `%w(...)`, etc.).
        // When set, body lines are skipped until the matching closer is found.
        let mut in_multiline_pct: Option<(u8, u8)> = None; // (opener, closer)
        let mut multiline_pct_depth: usize = 0;
        // Track backslash line continuations — continuation lines must not be
        // treated as inner-construct openers (e.g. `raise x \` / `  unless cond`).
        let mut prev_continued = false;
        // When a backslash-continuation line contains an assignment (`=`, `||=`, etc.),
        // the next continuation-line keyword (if/unless/case/begin) is an inline
        // conditional expression with a matching `end`, not a trailing modifier.
        let mut prev_continuation_was_assignment = false;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            let is_continuation = prev_continued;
            let continuation_is_assignment = prev_continuation_was_assignment;
            let line_trimmed_end = line.trim_end();
            prev_continued = line_trimmed_end.ends_with('\\');
            // Record if this continuation line has an assignment so the NEXT
            // continuation-line keyword can be treated as an inline block opener.
            prev_continuation_was_assignment = prev_continued && {
                let t_clean = line_trimmed_end.trim_end_matches('\\').trim_end();
                t_clean.ends_with('=') || t_clean.ends_with("||=")
                    || t_clean.ends_with("&&=") || t_clean.ends_with("+=")
                    || t_clean.contains(" = ") || t_clean.contains(" ||= ")
                    || t_clean.contains(" &&= ") || t_clean.contains(" += ")
            };

            // Skip heredoc body lines — def/end keywords inside heredocs are string content.
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // Skip multiline /regex/ body lines — content inside is not Ruby syntax.
            if in_multiline_regex {
                let tr = line.trim_start();
                if tr.starts_with('/') {
                    let after = tr[1..].trim_start_matches(|c: char| c.is_ascii_alphabetic());
                    if after.is_empty() || after.starts_with(' ') || after.starts_with('#')
                        || after.starts_with(';') || after.starts_with('\n')
                    {
                        in_multiline_regex = false;
                    }
                }
                continue;
            }

            // Skip multi-line percent literal body lines — keywords inside `%r{...}`,
            // `%w(...)`, etc. are not Ruby syntax.
            // When the closer is found on this line, fall through so that any Ruby
            // syntax AFTER the closer (e.g. `].map do |file|`) is still processed.
            if let Some((opener, closer)) = in_multiline_pct {
                let bytes = line.as_bytes();
                let len = bytes.len();
                let mut j = 0;
                while j < len {
                    if bytes[j] == b'\\' { j += 2; continue; }
                    if opener != closer && bytes[j] == opener {
                        multiline_pct_depth += 1;
                    } else if bytes[j] == closer {
                        if multiline_pct_depth > 0 {
                            multiline_pct_depth -= 1;
                        } else {
                            in_multiline_pct = None;
                            break;
                        }
                    }
                    j += 1;
                }
                // Only skip this line entirely if we're still inside the literal.
                if in_multiline_pct.is_some() {
                    continue;
                }
            }

            // Detect heredoc opener on this line — body starts on the NEXT line.
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: the opener line itself still contains real Ruby syntax.
            }

            if trimmed.starts_with('#') {
                continue;
            }

            let t = trimmed.trim();

            // Exclude one-liners and endless methods from all stack tracking.
            // Also catch well-known decorated defs like `helper_method def foo`,
            // `memoize def foo`, `deprecate def foo`, `attr_internal def foo`.
            let is_def_opener = !is_one_liner(t) && !is_endless_method(t) && (
                t.starts_with("def ") || t == "def"
                || t.starts_with("private def ") || t.starts_with("protected def ")
                || t.starts_with("private_class_method def ")
                || t.starts_with("module_function def ")
                || t.starts_with("class ") || t.starts_with("module ")
                || t.starts_with("helper_method def ")
                || t.starts_with("memoize def ")
                || t.starts_with("deprecate def ")
                || t.starts_with("attr_internal def ")
                || t.starts_with("public def ")
            );

            // `do` blocks on backslash-continuation lines are real block openers and must push a
            // frame. Only keyword-based openers (if/unless/while/until/begin/case) are skipped on
            // continuation lines because there they appear as trailing modifiers, not block openers.
            let is_do_block = !is_def_opener && !is_one_liner_block(t) && (
                ends_with_do_keyword(t)
                || has_unquoted_do_with_params(t)
                || has_do_before_comment(t)
            );
            let is_inline_keyword = t.starts_with("if ")
                || t.starts_with("unless ")
                || t.starts_with("while ")
                || t.starts_with("until ")
                || t == "begin"
                || t.starts_with("begin ")
                || t == "case" || t.starts_with("case ")
                // `!! case` / `! case` — case used as a value expression after a unary operator.
                || t.starts_with("!! case ") || t == "!! case"
                || t.starts_with("! case ") || t == "! case"
                // `return begin` / `break begin` / `next begin` / `yield begin` etc.
                // These open a begin/rescue/end block with a matching `end`.
                || t.ends_with(" begin");
            let is_inner_construct = !is_def_opener && !is_one_liner_block(t) && (
                is_do_block
                || (is_inline_keyword && (!is_continuation || continuation_is_assignment))
            );

            // Inline conditional/begin assignment: `x = if cond` / `x ||= if` / `x = begin` / etc.
            // The `end` that closes these should NOT be compared to the enclosing def.
            let has_inline_conditional = !is_def_opener && !is_inner_construct && (
                // Any assignment variant (=, ||=, &&=, +=, etc.) followed by if/unless/case.
                // Use `eq_keyword` to avoid matching operator-name suffixes like `[]=` in
                // `alias_method :x, :[]= unless method_defined?(:x)`.
                eq_keyword(t, "if") || eq_keyword(t, "unless") || eq_keyword(t, "case")
                || t.contains(" << if ") || t.ends_with(" << if")
                || t.contains(" << unless ") || t.ends_with(" << unless")
                || t.contains(" << case ") || t.ends_with(" << case")
                // Arithmetic/logical operators followed by if/unless/case
                // e.g. `acc + if index.even?`, `x | if cond`, `x || if cond`
                || t.contains(" + if ") || t.ends_with(" + if")
                || t.contains(" + unless ") || t.ends_with(" + unless")
                || t.contains(" + case ") || t.ends_with(" + case")
                || t.contains(" - if ") || t.ends_with(" - if")
                || t.contains(" - unless ") || t.ends_with(" - unless")
                || t.contains(" - case ") || t.ends_with(" - case")
                || t.contains(" * if ") || t.ends_with(" * if")
                || t.contains(" * unless ") || t.ends_with(" * unless")
                || t.contains(" * case ") || t.ends_with(" * case")
                || t.contains(" / if ") || t.ends_with(" / if")
                || t.contains(" / unless ") || t.ends_with(" / unless")
                || t.contains(" / case ") || t.ends_with(" / case")
                || t.contains(" | if ") || t.ends_with(" | if")
                || t.contains(" | unless ") || t.ends_with(" | unless")
                || t.contains(" | case ") || t.ends_with(" | case")
                || t.contains(" & if ") || t.ends_with(" & if")
                || t.contains(" & unless ") || t.ends_with(" & unless")
                || t.contains(" & case ") || t.ends_with(" & case")
                || t.contains(" || if ") || t.ends_with(" || if")
                || t.contains(" || unless ") || t.ends_with(" || unless")
                || t.contains(" || case ") || t.ends_with(" || case")
                || t.contains(" && if ") || t.ends_with(" && if")
                || t.contains(" && unless ") || t.ends_with(" && unless")
                || t.contains(" && case ") || t.ends_with(" && case")
                // `var = begin` / `var ||= begin` / `x || begin` inline begin/rescue/end block
                || t.ends_with("= begin") || t.ends_with("|| begin") || t.ends_with("&& begin")
                || t.contains("= begin ")
                // `(begin` — begin expression inside parentheses: `(begin...rescue...end)`
                || (t.contains("(begin") && { let p = t.find("(begin").unwrap(); let after = &t[p + 6..]; after.is_empty() || after.starts_with(' ') || after.starts_with('\n') })
                // `var = while cond` / `var = until cond` inline loop expression
                || t.contains("= while ") || t.ends_with("= while")
                || t.contains("= until ") || t.ends_with("= until")
            );

            if is_def_opener {
                stack.push((i, indent, true));
            } else if (is_inner_construct || has_inline_conditional) && !stack.is_empty() {
                stack.push((i, indent, false));
                if is_inner_construct {
                    let t_stripped = strip_trailing_comment(t);
                    // `if (expr = (begin` / `unless (expr = (begin` — keyword opener with an
                    // inline `(begin` block. The `(begin`'s `end` is extra, so push another frame.
                    let has_paren_begin = if let Some(pos) = t_stripped.find("(begin") {
                        let after = &t_stripped[pos + 6..];
                        after.is_empty() || !after.chars().next().map_or(false, |c| c.is_ascii_alphanumeric() || c == '_')
                    } else {
                        false
                    };
                    // `if begin` / `unless begin` — bare `begin` as the condition.
                    // Generates TWO `end`s: one closes `begin`, one closes `if/unless`. Push an
                    // extra frame so both `end`s are properly consumed.
                    // NOTE: `elsif begin` is NOT included — `elsif` does not add its own `end`,
                    // so only ONE frame is needed for the `begin` condition, which is already
                    // pushed above via `is_inner_construct`.
                    let bare_begin_condition =
                        t_stripped == "if begin" || t_stripped.starts_with("if begin ")
                        || t_stripped == "unless begin" || t_stripped.starts_with("unless begin ");
                    if has_paren_begin || bare_begin_condition {
                        stack.push((i, indent, false));
                    }
                }
            }

            // Match `end` followed by end-of-string or any non-identifier character
            // (covers `end`, `end,`, `end)`, `end]`, `end.`, `end `, etc.).
            // This mirrors the pattern in rescue_ensure_alignment.rs to prevent
            // `end,` / `end)` (do-block endings inside argument lists) from
            // leaving orphan frames on the stack and corrupting alignment checks.
            //
            // Also match `rescue ...; end` / `ensure; end` — the trailing `end`
            // of a rescue/ensure clause that closes a `begin` block opened on a
            // prior line (e.g. `begin` ... `rescue => e; end`).
            let is_rescue_ensure_end = (t.starts_with("rescue") || t.starts_with("ensure"))
                && (t.ends_with("; end") || t.ends_with(";end")
                    || t.ends_with(" end") || t.ends_with("\tend"));
            let is_end_token = is_rescue_ensure_end
                || t == "end"
                || (t.starts_with("end") && {
                    match t.as_bytes().get(3).copied() {
                        Some(c) => !c.is_ascii_alphanumeric() && c != b'_' && c != b':',
                        None => false,
                    }
                });

            if is_end_token {
                if let Some((_def_line, expected_indent, is_def)) = stack.pop() {
                    if is_def && indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`end` at indent {} does not match `def` at indent {}.",
                                indent, expected_indent
                            ),
                            range: TextRange::new(pos, pos + 3),
                            severity: Severity::Warning,
                        });
                    }
                    // If not is_def (inner construct), pop silently
                }
            }

            // Detect multi-line percent literal opener — body lines will be skipped.
            if in_multiline_pct.is_none() {
                if let Some((opener, closer)) = find_unclosed_percent_literal(line) {
                    in_multiline_pct = Some((opener, closer));
                    multiline_pct_depth = 0;
                }
            }

            // Detect multiline /regex/ opener — subsequent lines are regex body.
            if !in_multiline_regex && opens_unclosed_regex(line) {
                in_multiline_regex = true;
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubric_core::LintContext;
    use std::path::Path;

    fn check(src: &str) -> Vec<String> {
        let ctx = LintContext::new(Path::new("test.rb"), src);
        DefEndAlignment.check_source(&ctx)
            .into_iter()
            .map(|d| d.message)
            .collect()
    }

    // Regression: `def foo; 400 end` — one-liner with semicolon — must not push a frame.
    #[test]
    fn test_one_liner_def_with_semicolon() {
        let src = "class Foo\n  def http_status; 400 end\nend\n";
        assert!(check(src).is_empty(), "one-liner def should not push frame");
    }

    // Regression: `def self.schedule(*) yield end` — one-liner without semicolon.
    #[test]
    fn test_one_liner_def_without_semicolon() {
        let src = "class Foo\n  def self.schedule(*) yield end\n  def self.defer(*) yield end\nend\n";
        assert!(check(src).is_empty(), "one-liner def without semicolon should not push frame");
    }

    // Regression: eigenclass expression mid-line contains `; end)` which used
    // to match the old `contains("; end")` guard, preventing the outer do-block
    // frame from being pushed and leaving the stack corrupt.
    #[test]
    fn test_eigenclass_mid_line_does_not_prevent_do_block_frame() {
        let src = "def foo\n  (class << self; self; end).__send__(:bar) do |x|\n    x\n  end\nend\n";
        assert!(check(src).is_empty(), "eigenclass expression should not corrupt frame stack");
    }

    // Regression: bare `case` (no subject expression) was not in is_inner_construct,
    // causing its `end` to pop the enclosing def frame prematurely.
    #[test]
    fn test_bare_case_no_subject() {
        let src = "def foo\n  case\n  when 1 then :a\n  when 2 then :b\n  end\nend\n";
        assert!(check(src).is_empty(), "bare case should push inner-construct frame");
    }

    // Regression: `def foo x, y = default` — default param `=` must not be
    // mistaken for the endless-method body separator, which previously caused
    // the def frame to never be pushed and then a FP on the outer class end.
    #[test]
    fn test_method_with_default_param_not_endless() {
        let src = "class Foo\n  def parses pattern, mtype = :sinatra\n    pattern\n  end\nend\n";
        assert!(check(src).is_empty(), "default param should not trigger endless-method path");
    }

    // Sanity: a real endless method is still not pushed.
    #[test]
    fn test_endless_method_not_pushed() {
        let src = "class Foo\n  def double(x) = x * 2\nend\n";
        assert!(check(src).is_empty(), "endless method should not push frame");
    }

    // Regression: `get('/*')do` — `do` keyword directly after `)` without space.
    // Must be detected as a block opener so its frame is pushed and popped correctly.
    #[test]
    fn test_do_keyword_without_preceding_space() {
        let src = "class Foo\n  def bar\n    mock_app do\n      get('/*')do\n        true\n      end\n    end\n  end\nend\n";
        assert!(check(src).is_empty(), "do keyword without space should be detected as block opener");
    }

    // Sanity: genuine misalignment should still be reported.
    #[test]
    fn test_genuine_misalignment_still_reported() {
        let src = "def foo\n  42\n    end\n";
        assert!(!check(src).is_empty(), "misaligned end should be flagged");
    }
}

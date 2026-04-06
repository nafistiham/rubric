use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EndAlignment;

/// Returns true if `trimmed` is an endless method (`def foo = expr` / `def foo(x) = expr`).
/// Endless methods never have a matching `end` and should not be pushed onto the stack.
///
/// Distinguishes from default parameter syntax (`def foo x, y = default`):
/// - Endless: `def foo = expr` (no params) or `def foo(params) = expr` (all params in parens)
/// - NOT endless: `def foo x, y = default` — the `=` is inside a non-paren param list
///   (detected by `,` appearing at depth 0 before the `=`)
fn is_endless_method(trimmed: &str) -> bool {
    let def_pos = match trimmed.find("def ") {
        Some(p) if p <= 20 => p, // "def " near start of trimmed line
        _ => return false,
    };
    let after_def = &trimmed[def_pos + 4..]; // skip "def "
    let bytes = after_def.as_bytes();
    let n = bytes.len();
    let mut depth = 0i32;
    let mut saw_comma_at_depth0 = false;
    let mut i = 0;
    while i < n {
        match bytes[i] {
            b'(' => { depth += 1; }
            b')' => { depth -= 1; }
            // Comma at depth 0 means we have non-parenthesized params (e.g. `def foo x, y`).
            // An `=` after such a comma is a default value, not an endless-method `=`.
            b',' if depth == 0 => { saw_comma_at_depth0 = true; }
            // " = " at depth 0 (not "==" or "=>") indicates endless method —
            // but only if we haven't seen a depth-0 comma (which would mean default params).
            b' ' if depth == 0 && !saw_comma_at_depth0 && i + 2 < n
                && bytes[i + 1] == b'='
                && bytes[i + 2] != b'='
                && bytes[i + 2] != b'>' => {
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Returns true if `trimmed` is a one-liner — its opener and closer are on the same line.
/// Patterns:
///   - `class Foo; end` / `def foo; bar; end`      → ends with `; end`
///   - `def decode(*) { foo: "decoded" } end`      → ends with ` end` (no semicolon, body in braces)
///   - `class ::News; def self.has_many(*); end end`→ ends with ` end` and contains `; end`
/// All these should NOT be pushed onto the stack because their `end` is on the same line.
fn is_one_liner(trimmed: &str) -> bool {
    // Strip a trailing inline comment (`# ...`) before checking for `end`.
    // This handles `def foo() end # :nodoc:` — without stripping, the line
    // appears to end with `# :nodoc:` rather than ` end`.
    let code_part = strip_inline_comment_for_one_liner(trimmed);
    let bare = code_part.trim_end_matches(|c: char| c == ' ' || c == '\t');
    // Standard one-liner: `def foo; bar; end` / `class Foo; end`
    if bare.ends_with("; end") {
        return true;
    }
    // One-liner where body doesn't use semicolons: `def foo(*) { hash } end`
    // or nested: `class Foo; def bar; end end`
    // Any line that ends with ` end` (space before end) and has content before it.
    if bare.ends_with(" end") && bare.len() > 4 {
        // Exclude `def end` — here `end` is the method NAME, not the block closer.
        // E.g., `def end` defines a method named `end`; it is NOT a one-liner.
        let before_end = &bare[..bare.len() - 4]; // strip " end"
        if before_end == "def" || before_end.ends_with(" def") || before_end.ends_with("\tdef") {
            return false;
        }
        return true;
    }
    false
}

/// Strip a trailing inline comment from a line (simplified: finds the first
/// bare `#` not inside a string literal and not part of `#{...}` interpolation).
fn strip_inline_comment_for_one_liner(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(d) if bytes[i] == d => { in_str = None; }
            Some(_) => {}
            None if bytes[i] == b'\'' || bytes[i] == b'"' => { in_str = Some(bytes[i]); }
            None if bytes[i] == b'#' => {
                // `#{...}` is string interpolation, not a comment — skip over the `#`.
                if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    i += 1; // skip `#`; `{` will be processed next iteration
                } else {
                    return &s[..i];
                }
            }
            None => {}
        }
        i += 1;
    }
    s
}

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
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]); i += 1; continue;
            }
            None if bytes[i] == b'#' => break,
            None => {}
        }
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') { j += 1; }
            let quote = if j < len && matches!(bytes[j], b'\'' | b'"' | b'`') {
                let q = bytes[j]; j += 1; Some(q)
            } else { None };
            let start = j;
            if let Some(q) = quote {
                // Quoted heredoc: collect everything up to the closing quote
                while j < len && bytes[j] != q { j += 1; }
            } else {
                while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }
        i += 1;
    }
    None
}

/// Returns true if `trimmed` starts with `end` followed by a non-identifier character
/// (i.e., `end`, `end.foo`, `end,`, `end)`, `end]`, `end `, etc.).
/// This catches all valid Ruby `end` tokens regardless of what follows.
///
/// Note: `end:` is a valid Ruby hash key (Symbol `end:`) and is NOT an `end` token.
fn is_end_token(trimmed: &str) -> bool {
    if !trimmed.starts_with("end") {
        return false;
    }
    let rest = &trimmed[3..];
    if rest.is_empty() {
        return true; // bare "end"
    }
    let next = rest.as_bytes()[0];
    // end followed by non-alphanumeric and non-underscore = valid end token,
    // EXCEPT `end:` which is a Symbol hash key, not a Ruby `end` keyword.
    if next == b':' {
        return false;
    }
    !next.is_ascii_alphanumeric() && next != b'_'
}

/// Check whether a trimmed line contains an inline block opener of the form:
/// - `lhs = if/unless/case/begin` (assignment + keyword)
/// - `arr << if cond` (shovel operator + keyword)
/// - `(if ...`, `(unless ...`, `(case ...` (parenthesised inline conditional)
/// - `+ if`, `- if`, `| if`, `|| if`, `&& if`, etc. (arithmetic/logical operator + keyword)
///
/// These patterns open a multi-line `if/unless/case/begin` block mid-line whose
/// `end` alignment is NOT checked by Layout/EndAlignment (rubocop skips inline ends).
fn has_inline_block_opener(trimmed: &str) -> bool {
    // Check for `= if`, `= unless`, `= case`, `= while`, `= until` (with one or more spaces)
    for kw in &["if", "unless", "case", "while", "until"] {
        if contains_assign_kw(trimmed, kw) {
            return true;
        }
    }
    // `= begin` / `|| begin` / `&& begin` at end of line or followed by a space
    // but NOT `= begin_something` (method name starting with "begin").
    for prefix in &["= begin", "|| begin", "&& begin"] {
        if let Some(pos) = trimmed.find(prefix) {
            let after = &trimmed[pos + prefix.len()..];
            // Word-boundary: must be followed by space, EOL, or `#` (comment)
            if after.is_empty() || after.starts_with(' ') || after.starts_with('#') {
                return true;
            }
        }
    }
    // `return begin` — begin expression returned from a method
    if trimmed.contains("return begin") {
        let after = &trimmed[trimmed.find("return begin").unwrap() + 12..];
        if after.is_empty() || after.starts_with(' ') || after.starts_with('#') {
            return true;
        }
    }
    // `<< if`, `<< unless`, `<< case`, `<< begin`
    for kw in &["if", "unless", "case", "begin"] {
        let needle = format!("<< {}", kw);
        if let Some(pos) = trimmed.find(&needle) {
            let after = &trimmed[pos + needle.len()..];
            if after.is_empty() || after.starts_with(' ') {
                return true;
            }
        }
    }
    // `(if ...`, `(unless ...`, `(case ...`, `(begin` — parenthesised inline conditional/begin
    for kw in &["if ", "unless ", "case ", "begin"] {
        let needle = if kw.ends_with(' ') { format!("({}", kw) } else { format!("({})", kw.trim_end()) };
        // For `(begin`, allow followed by space, newline, rescue, or nothing
        if *kw == "begin" {
            if let Some(pos) = trimmed.find("(begin") {
                let after = &trimmed[pos + 6..];
                if after.is_empty() || after.starts_with(' ') || after.starts_with('\n') {
                    return true;
                }
            }
        } else if trimmed.contains(&format!("({}", kw)) {
            return true;
        }
        let _ = needle;
    }
    // `!! case expr`, `! case expr` — boolean coercion of a case expression.
    // `return case expr` — case expression returned from a method.
    // These open an inline case block whose `end` alignment is not checked.
    if trimmed.contains("!! case") || trimmed.contains("! case ") {
        return true;
    }
    if trimmed.contains("return case ") || trimmed.ends_with("return case") {
        return true;
    }
    // `elsif begin` / `elsif (begin` / `elsif (expr = (begin` — begin as the elsif condition.
    if trimmed.starts_with("elsif begin") || trimmed.starts_with("elsif (begin")
        || (trimmed.starts_with("elsif (") && trimmed.contains("(begin"))
    {
        return true;
    }
    // Trailing-modifier `if begin` / `unless begin`:
    // e.g., `next if begin ... rescue ... end`
    // The `begin...end` block serves as the condition of the trailing `if`/`unless`.
    for pattern in &[" if begin", " unless begin"] {
        if let Some(pos) = trimmed.find(pattern) {
            if pos > 0 {
                let after = &trimmed[pos + pattern.len()..];
                if after.is_empty() || after.starts_with(' ') || after.starts_with('#') {
                    return true;
                }
            }
        }
    }

    // Arithmetic and logical operators before if/unless/case:
    // e.g., `acc + if cond`, `val | if cond`, `x || if cond`, `x && if cond`
    // We look for these specific operator patterns followed by the keyword with a word boundary.
    for op in &[" + ", " - ", " * ", " / ", " | ", " & ", " || ", " && "] {
        for kw in &["if", "unless", "case"] {
            let needle = format!("{}{} ", op, kw);
            if trimmed.contains(&needle) {
                return true;
            }
            // keyword at end of line (body on next line)
            let needle_eol = format!("{}{}", op, kw);
            if trimmed.ends_with(&needle_eol) {
                return true;
            }
        }
    }
    false
}

/// Returns true if `trimmed` contains `= kw` (with any amount of whitespace between)
/// where `kw` is followed by a space or is at end of line.
fn contains_assign_kw(trimmed: &str, kw: &str) -> bool {
    // Look for `=` followed by optional spaces followed by `kw` followed by space or EOL
    let bytes = trimmed.as_bytes();
    let n = bytes.len();
    let kw_bytes = kw.as_bytes();
    let kw_len = kw_bytes.len();

    let mut i = 0;
    while i < n {
        if bytes[i] == b'=' && (i == 0 || {
            let prev = bytes[i - 1];
            // Exclude: `!=`, `<=`, `>=`, `[]=` (operators), and setter method names like `method=`
            prev != b'!' && prev != b'<' && prev != b'>' && prev != b']'
                && !prev.is_ascii_alphanumeric() && prev != b'_'
        }) {
            // Skip past optional `=` in `==`, `=>`, `=~`
            if i + 1 < n && (bytes[i + 1] == b'=' || bytes[i + 1] == b'>' || bytes[i + 1] == b'~') {
                i += 1;
                continue;
            }
            // Skip spaces after `=`
            let mut j = i + 1;
            while j < n && bytes[j] == b' ' { j += 1; }
            // Check if kw matches at position j
            if j + kw_len <= n && &bytes[j..j + kw_len] == kw_bytes {
                let after_kw = j + kw_len;
                if after_kw >= n || bytes[after_kw] == b' ' || bytes[after_kw] == b'\n' {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Returns true if `trimmed` contains a `do` block pattern (` do |`, ` do|`, ` do #`,
/// ` do;`) that is **not** inside a string literal ending on the same line.
///
/// Strategy: find the last occurrence of each pattern, then check if the
/// remainder of the line (after the pattern) ends with a closing string
/// delimiter `"` or `'` — if it does, the pattern is inside a string argument.
/// This handles cases like `puts "cascade do |t|"` (inside string, skip) vs
/// `.gsub(/regex/) do |match|` (outside string, detect).
fn has_do_pattern_outside_string(trimmed: &str) -> bool {
    for pattern in &[" do |", " do|", ")do |", ")do|", "]do |", "]do|", "}do |", "}do|"] {
        if let Some(pos) = trimmed.rfind(pattern) {
            let after = &trimmed[pos + pattern.len()..];
            let code_after = strip_inline_comment_for_one_liner(after);
            let bare_after = code_after.trim_end();
            // If the tail ends with a string quote, the `do |` is inside a string.
            if bare_after.ends_with('"') || bare_after.ends_with('\'') {
                continue;
            }
            // For `do |params|`, check that the content after the CLOSING `|` is
            // empty or just a comment — real block params end the line after `|`.
            // If there's significant content after the closing `|`, the pattern is
            // inside a regex or string (e.g. `assert_match(/...do |x| %>/, y)`).
            if let Some(pipe_pos) = bare_after.find('|') {
                let after_params = bare_after[pipe_pos + 1..].trim_start();
                if !after_params.is_empty() && !after_params.starts_with('#') {
                    continue; // content after params — inside regex/string
                }
            }
            return true;
        }
    }
    for pattern in &[" do #", " do;"] {
        if let Some(pos) = trimmed.rfind(pattern) {
            let after = &trimmed[pos + pattern.len()..];
            let code_after = strip_inline_comment_for_one_liner(after);
            let bare_after = code_after.trim_end();
            if bare_after.ends_with('"') || bare_after.ends_with('\'') {
                continue;
            }
            return true;
        }
    }
    // `do` followed by 2+ spaces then `#` (e.g., `foo.bar do  # comment`).
    // The single-space case ` do #` is already handled above; this catches double-space etc.
    if let Some(pos) = trimmed.rfind(" do") {
        let after = &trimmed[pos + 3..];
        if !after.is_empty() {
            let after_trim = after.trim_start_matches(' ');
            if after_trim.starts_with('#') {
                return true;
            }
        }
    }
    false
}

/// Returns the unclosed paren depth after scanning a line for multi-line percent literals
/// (`%w(...)`, `%i(...)`, `%W(...)`, `%I(...)`, `%(...)`, `%q(...)`, `%Q(...)`).
/// Returns 0 if no unclosed literal is found.
fn opening_pct_literal_depth(line: &str) -> i32 {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i + 1 < n {
        if bytes[i] == b'%' && (
            // %w( %W( %i( %I( — word/symbol arrays
            (i + 2 < n && matches!(bytes[i + 1], b'w' | b'W' | b'i' | b'I') && bytes[i + 2] == b'(')
            // %( — general string literal (equivalent to double-quoted string)
            || bytes[i + 1] == b'('
            // %q( %Q( — quoted string literals
            || (i + 2 < n && matches!(bytes[i + 1], b'q' | b'Q') && bytes[i + 2] == b'(')
        ) {
            // Find the position of the opening `(`
            let paren_pos = if bytes[i + 1] == b'(' { i + 1 } else { i + 2 };
            if paren_pos >= n { i += 1; continue; }
            // Count the net unmatched parens from the `(` onwards.
            let mut depth = 0i32;
            for &b in &bytes[paren_pos..] {
                match b {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if depth > 0 {
                return depth;
            }
        }
        i += 1;
    }
    0
}

impl Rule for EndAlignment {
    fn name(&self) -> &'static str {
        "Layout/EndAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Unified stack: (indent, check_alignment)
        // check_alignment=true  → block opener at line start (if/while/def/class/begin)
        //                         whose `end` alignment we WANT to check
        // check_alignment=false → do-block openers or inline openers (= if / = begin / etc.)
        //                         whose `end` alignment we do NOT check
        //                         (rubocop's EndAlignment does not check `do`-block ends;
        //                          those are covered by Layout/BlockAlignment)
        let mut stack: Vec<(usize, bool)> = Vec::new();

        // Heredoc tracking: when inside a heredoc, skip all lines until the terminator.
        let mut in_heredoc: Option<String> = None;

        // Percent-word-literal tracking: when inside a multi-line `%w(...)`, `%i(...)`,
        // `%W(...)`, or `%I(...)` literal, skip keyword detection — the "words" inside are
        // string content, not Ruby syntax.
        let mut pct_literal_depth: i32 = 0;

        // Percent-regex literal tracking: when inside a multi-line `%r{...}`, skip lines
        // since `end` inside a regex is pattern content, not a Ruby `end` keyword.
        let mut pct_regex_depth: i32 = 0;

        // Bracket-delimited percent literal tracking: `%w[...]`, `%i[...]`, `%w{...}`, etc.
        // The opener/closer pair is tracked so nested brackets are handled correctly.
        let mut pct_bracket_depth: i32 = 0;
        let mut pct_bracket_opener: u8 = 0;
        let mut pct_bracket_closer: u8 = 0;

        // Continuation line tracking: if the previous non-comment, non-blank line ends
        // with `\`, the current line is a continuation and should NOT be treated as a
        // new block opener (e.g., `unless` on a continuation is not a new `unless` block).
        let mut prev_line_continues = false;
        // Track whether the continuation line contained an assignment (`= \`, `||= \`, etc.).
        // When true and the NEXT line starts with `if`/`unless`, that keyword opens an inline
        // block (like `x = \ if cond`) rather than being a trailing modifier.
        let mut prev_continuation_had_assign = false;

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            // Skip heredoc body lines — Ruby keywords inside heredocs are string content.
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                i += 1;
                continue;
            }

            // Detect heredoc opener on this line — body starts on the NEXT line.
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: the opener line itself still contains real Ruby syntax.
            }

            // Skip percent-regex body lines (`%r{...}` spanning multiple lines).
            // `end` inside a regex literal is pattern content, not a Ruby keyword.
            if pct_regex_depth > 0 {
                for &b in line.as_bytes() {
                    match b {
                        b'{' => pct_regex_depth += 1,
                        b'}' => pct_regex_depth -= 1,
                        _ => {}
                    }
                }
                if pct_regex_depth > 0 {
                    i += 1;
                    continue;
                }
                // depth dropped to 0: fall through to process the closing line
            }
            // Detect multi-line %r{...} regex literals opening on this line.
            if line.contains("%r{") {
                let mut depth: i32 = 0;
                let bytes = line.as_bytes();
                let start = line.find("%r{").unwrap() + 2; // position of `{`
                for &b in &bytes[start..] {
                    match b {
                        b'{' => depth += 1,
                        b'}' => { depth -= 1; if depth == 0 { break; } }
                        _ => {}
                    }
                }
                if depth > 0 {
                    pct_regex_depth = depth;
                }
            }

            // Skip bracket-delimited percent literal body lines (`%w[...]`, `%i[...]`, etc.).
            if pct_bracket_depth > 0 {
                for &b in line.as_bytes() {
                    if b == pct_bracket_opener { pct_bracket_depth += 1; }
                    else if b == pct_bracket_closer {
                        pct_bracket_depth -= 1;
                        if pct_bracket_depth == 0 { break; }
                    }
                }
                if pct_bracket_depth > 0 {
                    i += 1;
                    continue;
                }
                // depth dropped to 0: fall through to process the closing line
            }
            // Detect bracket-delimited percent literals opening on this line.
            // Handles `%w[...]`, `%W[...]`, `%i[...]`, `%I[...]`, `%w{...}`, `%i{...}`,
            // `%w<...>`, and also bare `%[...]`, `%{...}` string literals.
            if pct_bracket_depth == 0 {
                let bytes = line.as_bytes();
                let n_bytes = bytes.len();
                let mut j = 0;
                while j + 1 < n_bytes {
                    if bytes[j] == b'%' {
                        let (type_char, bracket_pos) = if j + 2 < n_bytes
                            && matches!(bytes[j + 1], b'w' | b'W' | b'i' | b'I' | b'q' | b'Q' | b'x' | b's')
                        {
                            (bytes[j + 1], j + 2)
                        } else {
                            (0u8, j + 1)
                        };
                        let bp = bracket_pos;
                        if bp < n_bytes {
                            let (opener, closer) = match bytes[bp] {
                                b'[' => (b'[', b']'),
                                b'{' => (b'{', b'}'),
                                b'<' => (b'<', b'>'),
                                _ => (0, 0),
                            };
                            let _ = type_char;
                            if opener != 0 {
                                let mut depth = 0i32;
                                let mut closed = false;
                                let mut k = bp;
                                while k < n_bytes {
                                    if bytes[k] == opener { depth += 1; }
                                    else if bytes[k] == closer {
                                        depth -= 1;
                                        if depth == 0 { closed = true; break; }
                                    }
                                    k += 1;
                                }
                                if !closed && depth > 0 {
                                    pct_bracket_depth = depth;
                                    pct_bracket_opener = opener;
                                    pct_bracket_closer = closer;
                                    break;
                                }
                                j = if closed { k + 1 } else { k };
                                continue;
                            }
                        }
                    }
                    j += 1;
                }
            }

            // Skip percent-literal body lines (`%w(...)`, `%i(...)`, etc. spanning multiple lines).
            // Ruby keywords inside these literals are string content, not openers.
            if pct_literal_depth > 0 {
                for &b in line.as_bytes() {
                    match b {
                        b'(' => pct_literal_depth += 1,
                        b')' => pct_literal_depth -= 1,
                        _ => {}
                    }
                }
                if pct_literal_depth > 0 {
                    i += 1;
                    continue;
                }
                // depth dropped to 0 on this line: fall through so that content AFTER
                // the closing `)` (e.g. `.each do |method|`) is processed normally.
            }
            // Detect multi-line percent literals opening on this line.
            pct_literal_depth = opening_pct_literal_depth(line);

            // Skip comments — also do not update prev_line_continues for comment lines.
            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // A line is a continuation if the PREVIOUS non-comment, non-blank line
            // ended with a backslash `\`.
            let is_continuation = prev_line_continues;
            let continuation_had_assign = prev_continuation_had_assign;

            // Update continuation state for the next iteration.
            let bare = trimmed.trim_end();
            prev_line_continues = bare.ends_with('\\');
            prev_continuation_had_assign = if prev_line_continues {
                // Check if the code before `\` ends with an assignment operator.
                let code_before = bare.trim_end_matches('\\').trim_end();
                code_before.ends_with(" =")
                    || code_before.ends_with("||=")
                    || code_before.ends_with("&&=")
                    || code_before.ends_with("+=")
                    || code_before.ends_with("-=")
            } else {
                false
            };

            // Detect block/construct openers at the start of the trimmed line.
            // Exclude: one-liners (`class Foo; end`), endless methods (`def foo = expr`),
            //          and continuation lines (a `unless` on a continuation line is not
            //          a new block opener but part of the previous line's expression).
            //
            // IMPORTANT: `do` blocks are pushed with check=false (no alignment check).
            // Rubocop's Layout/EndAlignment does NOT check do-block end alignment;
            // that's the job of Layout/BlockAlignment. Pushing do-blocks as check=false
            // ensures their `end` tokens are consumed from the stack without generating
            // false diagnostics.
            //
            // Detect `do` at end of line. Use the raw `trimmed` (not comment-stripped) so
            // Check if the line (raw) ends with ` do`.
            let raw_do_at_end = trimmed == "do" || trimmed.ends_with(" do");
            // Exclude cases where ` do` at end-of-line is inside a comment:
            //   `some_code # prose that mentions do`
            // but NOT `#` inside a string literal or `#method` patterns.
            // Uses a string-aware forward scan — only ` #` preceded by a space is a
            // real comment start; `#method` (no preceding space) is a method reference.
            let is_comment_ending_do = raw_do_at_end && {
                let bytes = trimmed.as_bytes();
                let n = bytes.len();
                let mut in_str: Option<u8> = None;
                let mut real_comment_text: Option<&str> = None;
                let mut j = 0;
                while j < n {
                    match in_str {
                        Some(_) if bytes[j] == b'\\' => { j += 2; continue; }
                        Some(d) if bytes[j] == d => { in_str = None; j += 1; continue; }
                        Some(_) => { j += 1; continue; }
                        None if bytes[j] == b'"' || bytes[j] == b'\'' => {
                            in_str = Some(bytes[j]); j += 1; continue;
                        }
                        None if bytes[j] == b'#' => {
                            // `#{...}` is interpolation, not a comment.
                            if j + 1 < n && bytes[j + 1] == b'{' { j += 2; continue; }
                            // ` # ` pattern (space before AND after `#`) → real comment.
                            // Requiring space after `#` prevents `#<Obj>` or `#method`
                            // patterns (common in regex literals and error messages) from
                            // being treated as comments.
                            if j > 0 && bytes[j - 1] == b' '
                                && (j + 1 >= n || bytes[j + 1] == b' ')
                            {
                                real_comment_text = Some(&trimmed[j + 1..]);
                                break;
                            }
                            // `#method`, `#<obj>`, `:sub#method` — not a comment.
                            j += 1; continue;
                        }
                        None => {}
                    }
                    j += 1;
                }
                if let Some(ct) = real_comment_text {
                    let ct = ct.trim_start();
                    ct == "do" || ct.ends_with(" do")
                } else {
                    false
                }
            };
            let is_do_block = !is_continuation && !is_one_liner(trimmed) && (
                (raw_do_at_end && !is_comment_ending_do)
                // Use string-aware scan for "do |", "do|", "do #", "do;" patterns so that
                // these substrings inside string literals (e.g. `puts "cascade do |t|"`)
                // do not falsely trigger a do-block frame.
                || has_do_pattern_outside_string(trimmed)
            );

            let is_keyword_opener = !is_continuation && !is_one_liner(trimmed) && !is_endless_method(trimmed) && !is_do_block && (
                trimmed.starts_with("def ") || trimmed == "def"
                || trimmed.starts_with("private def ") || trimmed.starts_with("protected def ")
                || trimmed.starts_with("private_class_method def ") || trimmed.starts_with("public_class_method def ")
                || trimmed.starts_with("module_function def ")
                // Any single-word modifier before `def`: `helper_method def`, `memoize def`, etc.
                // Detected by: word chars only before " def " (no spaces/operators in the modifier).
                || {
                    let is_modifier_def = if let Some(p) = trimmed.find(" def ") {
                        trimmed[..p].chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    } else {
                        false
                    };
                    is_modifier_def
                }
                || trimmed.starts_with("class ") || trimmed.starts_with("module ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed == "begin"
                || trimmed.starts_with("begin ") || trimmed.starts_with("case ")
                || trimmed == "case"  // bare caseless case expression
            );

            // Detect inline if/unless/case/begin assignments that open a block mid-line
            // Pattern: something = if condition  (or unless/case/begin, with any = variant: =, ||=, &&=, etc.)
            // Also: arithmetic/logical operators before if/unless/case (e.g., `acc + if cond`)
            let is_inline_opener = !is_continuation && !is_keyword_opener && !is_do_block && has_inline_block_opener(trimmed);

            // When the previous line ends with `\` (backslash continuation) and the current
            // line opens a keyword block or do-block, track it as an inline opener (check=false).
            // E.g.:
            //   `options = \` + `  case last`      — inline case expression
            //   `test "very long name" \` + `"cont" do` — multi-line method call with do-block
            // Rubocop does not check end alignment for these because the block is part of a
            // wrapped expression.
            //
            // `if`/`unless` are included ONLY when the previous continuation line contained
            // an assignment operator (`= \`, `||= \`, etc.), which indicates the keyword
            // opens an inline block expression (not a trailing modifier like `raise ... \
            //   unless cond`).
            let is_continuation_keyword = is_continuation && !is_one_liner(trimmed) && (
                trimmed.starts_with("case ") || trimmed == "case"
                || trimmed == "begin" || trimmed.starts_with("begin ")
                // Continuation line ending with `do` — method call split across lines with `\`
                || (raw_do_at_end && !is_comment_ending_do)
                || has_do_pattern_outside_string(trimmed)
                // `if`/`unless` on a continuation from an assignment line open inline blocks:
                // `x = \` + `  if cond` — the `if` is part of the `= if cond ... end` expression.
                || (continuation_had_assign && (
                    trimmed.starts_with("if ") || trimmed == "if"
                    || trimmed.starts_with("unless ") || trimmed == "unless"
                ))
            );

            if is_keyword_opener {
                stack.push((indent, true));
                // `if begin` / `unless begin` / `if (expr = (begin` — `begin` serves as the
                // condition expression; its matching `end` is "inline" and should not
                // be alignment-checked independently.
                let cond_str = strip_inline_comment_for_one_liner(trimmed);
                let is_begin_condition = cond_str == "if begin"
                    || cond_str.starts_with("if begin ")
                    || cond_str == "unless begin"
                    || cond_str.starts_with("unless begin ")
                    // `if (expr = (begin` etc. — begin inside parentheses as the condition
                    || {
                        if let Some(pos) = cond_str.find("(begin") {
                            let after = &cond_str[pos + 6..];
                            after.is_empty() || !after.chars().next().map_or(false, |c| c.is_ascii_alphanumeric() || c == '_')
                        } else {
                            false
                        }
                    };
                if is_begin_condition {
                    stack.push((indent, false)); // inline begin frame — end is not alignment-checked
                }
            } else if is_do_block {
                // Push with check=false: we track the frame for correct end-consumption,
                // but do NOT flag misaligned ends.
                stack.push((indent, false));
            } else if is_inline_opener || is_continuation_keyword {
                stack.push((indent, false));
            }

            // Detect end tokens: `end` followed by any non-identifier character
            // (covers: end, end.foo, end,, end), end], end if, etc.)
            // Also detect `rescue ...; end` / `ensure ...; end` — one-liner rescue/ensure
            // clauses that close the surrounding begin/def/do block on the same line.
            let is_end = is_end_token(trimmed) || {
                let code = strip_inline_comment_for_one_liner(trimmed);
                let bare_code = code.trim_end();
                (trimmed.starts_with("rescue") || trimmed.starts_with("ensure"))
                    && bare_code.ends_with("; end")
            };

            if is_end {
                if let Some((expected_indent, check)) = stack.pop() {
                    if check && indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i];
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`end` indentation ({}) does not match its opener ({}).",
                                indent, expected_indent
                            ),
                            range: TextRange::new(
                                line_start + indent as u32,
                                line_start + indent as u32 + 3,
                            ),
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

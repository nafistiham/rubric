use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NestedMethodDefinition;

/// Strips a trailing inline `# comment` from a line so checks like
/// `ends_with(" end")` work even when `end` is followed by `# :nodoc:`.
fn strip_trailing_comment(s: &str) -> &str {
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
                // `#{` is string interpolation, not a comment.
                if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    i += 1;
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

/// Tracks what kind of block is currently open.
#[derive(Debug, PartialEq)]
enum FrameKind {
    /// An opened `def ... end` block.
    Def,
    /// Any other block: do-block, class, module, if, unless, while, until,
    /// for, case, begin, or a `{...}` brace block (Class.new { }, etc.).
    Other,
}

/// Returns `true` when the trimmed line is an endless method definition,
/// i.e. `def name = expr` or `def name(params) = expr`.
///
/// Detection strategy: find `def ` at or near the start of the trimmed line,
/// then scan the remainder for a bare ` = ` (not `==`, not `=>`) at
/// parenthesis depth 0.
fn is_endless_method(t: &str) -> bool {
    let def_pos = match t.find("def ") {
        // Allow `def ` to appear at position 0–20 (leading spaces already
        // stripped by the caller; small cap is a safety guard).
        Some(p) if p <= 20 => p,
        _ => return false,
    };
    let after = &t[def_pos + 4..];
    let bytes = after.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b' ' if depth == 0
                && i + 2 < bytes.len()
                && bytes[i + 1] == b'='
                // Must not be `==` or `=>`
                && bytes[i + 2] != b'='
                && bytes[i + 2] != b'>' =>
            {
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Returns `true` when the trimmed line is a single-line method definition that
/// opens and closes on the same line, e.g. `def setup; body; end`.
///
/// These one-liners never push to the stack because their `end` token is on
/// the same line as the `def` — the line-by-line scanner would otherwise miss
/// the closing `end` and leave a frame permanently on the stack.
fn is_one_liner_def(t: &str) -> bool {
    // "; end" pattern: `def foo; body; end`
    if t.contains("; end") || t.contains(";end") {
        return true;
    }
    // Strip trailing inline comment before checking ` end` suffix so that
    // `def foo() end # :nodoc:` is correctly detected as a one-liner.
    let bare = strip_trailing_comment(t).trim_end();
    // Any def line ending with ` end` — covers `def header; {} end`,
    // `def encode(*) 'str' end`, `def body; "" end`, etc.
    bare.starts_with("def ") && (bare.ends_with(" end") || bare.ends_with("\tend"))
}

/// Returns `true` when the trimmed line is a one-liner non-def block whose
/// opening keyword and closing `end` appear on the same line, e.g.
/// `until token = scan || @scanner.eos?; end` or `while true; break; end`.
///
/// These must NOT push a frame because the line-by-line scanner would never
/// see a standalone `end` line to pop them, leaving a phantom frame.
fn is_one_liner_block(t: &str) -> bool {
    let t = t.trim_end_matches(|c: char| c == ' ' || c == '\t');
    t.ends_with("; end") || t.ends_with(";end") || t.ends_with(" end") || t.ends_with("\tend")
}

/// Returns `true` when `pattern` appears in `s` outside of string literals
/// (`"..."`, `'...'`, `%{...}`, `%(...)`).
fn contains_outside_string(s: &str, pattern: &str) -> bool {
    let bytes = s.as_bytes();
    let pat = pattern.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut pct_depth: i32 = 0;  // depth inside %{...} or %(...)
    let mut i = 0;
    while i < bytes.len() {
        if let Some(delim) = in_str {
            if bytes[i] == b'\\' { i += 2; continue; }
            if bytes[i] == delim { in_str = None; }
            i += 1;
            continue;
        }
        if pct_depth > 0 {
            match bytes[i] {
                b'{' | b'(' => pct_depth += 1,
                b'}' | b')' => { pct_depth -= 1; }
                _ => {}
            }
            i += 1;
            continue;
        }
        // Check for percent literals: %{, %(, %w{, %w(, etc.
        if bytes[i] == b'%' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if next == b'{' || next == b'(' {
                pct_depth = 1; i += 2; continue;
            }
            // %w{, %i{, %W{, %r{, etc.
            if i + 2 < bytes.len() && (next == b'{' || bytes[i + 2] == b'{' || bytes[i + 2] == b'(') {
                pct_depth = 1; i += 3; continue;
            }
        }
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            in_str = Some(bytes[i]); i += 1; continue;
        }
        // Match the pattern at current position
        if i + pat.len() <= bytes.len() && &bytes[i..i + pat.len()] == pat {
            return true;
        }
        i += 1;
    }
    false
}

/// Returns `true` when the line contains ` do |params|` as a real block opener —
/// i.e. the ` do |` appears outside of string/percent literals AND the content
/// after the CLOSING `|` is empty or a comment (not `%>/)` which would indicate
/// the `do |` is inside a regex literal or template string).
fn has_do_params_block(t: &str) -> bool {
    for pattern in &[" do |", " do|"] {
        // Find the pattern outside string literals using our scanner.
        if !contains_outside_string(t, pattern) {
            continue;
        }
        // Find where the pattern ends and check what's after the closing `|`.
        if let Some(pos) = t.rfind(pattern) {
            let after_prefix = &t[pos + pattern.len()..];
            let code_after = strip_trailing_comment(after_prefix).trim_end();
            // Skip if the region ends with a string delimiter (pattern inside string).
            if code_after.ends_with('"') || code_after.ends_with('\'') {
                continue;
            }
            // Find the closing `|` of the params list.
            if let Some(pipe_pos) = code_after.find('|') {
                let after_params = code_after[pipe_pos + 1..].trim_start();
                // If there is non-comment content after the closing `|`, this
                // `do |` is inside a larger expression (regex, template, etc.).
                if !after_params.is_empty() && !after_params.starts_with('#') {
                    continue;
                }
            }
            return true;
        }
    }
    false
}

/// Returns `true` when the trimmed line opens a non-def block that requires a
/// matching `end`: do-blocks, class, module, if, unless, while, until, for,
/// case, begin.
///
/// Single-line class/module (`class Foo; end`) are excluded because their
/// closing `end` appears on the same line.
fn opens_other_block(t: &str) -> bool {
    // Strip trailing comment before checking `ends_with(" do")` to avoid
    // matching comments like `# silence libxml, exceptions will do`.
    let bare = strip_trailing_comment(t).trim_end();
    if bare == "do" || bare.ends_with(" do") {
        return true;
    }

    // ` do |params|` — only when `do |` appears outside string literals and
    // the content after the CLOSING `|` is empty or a comment (not `%>/)` etc.).
    if has_do_params_block(t) {
        return true;
    }
    // ` do #` / ` do;` — do-block with immediate comment or statement
    if t.contains(" do #") || t.contains(" do;") {
        return true;
    }

    // Keyword-based openers
    let keyword_match = t.starts_with("if ")
        || t.starts_with("unless ")
        || t.starts_with("while ")
        || t.starts_with("until ")
        || t.starts_with("for ")
        || t.starts_with("case ")
        || t == "begin"
        || t.starts_with("begin ");

    if keyword_match {
        return true;
    }

    // class / module — but NOT one-liners like `class Foo; end`
    if t.starts_with("class ") || t.starts_with("module ") {
        // One-liner: contains `; end` (with or without trailing content)
        if t.contains("; end") || t.contains(";end") {
            return false;
        }
        return true;
    }

    false
}

/// Extract the heredoc terminator word from a line that opens a heredoc.
///
/// Handles `<<~WORD`, `<<-WORD`, and `<<WORD` (bare). Returns `None` if
/// no heredoc opener is found on the line.
///
/// The terminator is the unquoted, stripped identifier after `<<`, `<<-`,
/// or `<<~`.  Quoted heredocs (e.g. `<<"WORD"` or `<<'WORD'`) are also
/// handled by stripping the surrounding quote characters.
fn heredoc_terminator(line: &str) -> Option<String> {
    // Find `<<` in the line.
    let pos = line.find("<<")?;
    let rest = &line[pos + 2..];

    // Strip optional `-` or `~` sigil.
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    let rest = rest.strip_prefix('~').unwrap_or(rest);

    // Strip optional surrounding quotes.
    let rest = if (rest.starts_with('"') && rest.contains('"'))
        || (rest.starts_with('\'') && rest.contains('\''))
        || (rest.starts_with('`') && rest.contains('`'))
    {
        &rest[1..]
    } else {
        rest
    };

    // Collect the identifier (letters, digits, underscores).
    let word: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if word.is_empty() {
        None
    } else {
        Some(word)
    }
}

impl Rule for NestedMethodDefinition {
    fn name(&self) -> &'static str {
        "Lint/NestedMethodDefinition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        // Frame stack: each opened block (def or other) pushes a frame.
        // A `def` is only flagged as nested when the TOP frame is `Def`
        // (i.e. directly inside another def, not inside an intervening block).
        let mut stack: Vec<FrameKind> = Vec::new();
        // When `Some(word)`, we are inside a heredoc whose terminator is `word`.
        let mut heredoc_terminator_word: Option<String> = None;

        for i in 0..n {
            let raw = &lines[i];
            let trimmed = raw.trim_start();
            let t = trimmed.trim();

            // If we are inside a heredoc, look for the terminator line.
            if let Some(ref term) = heredoc_terminator_word {
                if t == term.as_str() {
                    heredoc_terminator_word = None;
                }
                // Either way, skip this line for def/end tracking.
                continue;
            }

            if t.starts_with('#') {
                continue;
            }

            // Check whether this line opens a heredoc; if so, record the
            // terminator so subsequent lines are skipped.
            if let Some(term) = heredoc_terminator(raw) {
                heredoc_terminator_word = Some(term);
                // The opening line itself is still valid Ruby — fall through
                // to check for `def` on the same line (e.g. `def foo <<~MSG`
                // is unusual but technically valid; for safety we do NOT skip
                // the opener line for def/end accounting).
            }

            // `rescue ... ; end` / `ensure ... ; end` — these are one-liner
            // rescue/ensure clauses that also close the enclosing begin block.
            // Pop the matching begin frame without any push.
            {
                let tc = strip_trailing_comment(t).trim_end();
                let is_rescue_end = (t.starts_with("rescue") || t.starts_with("ensure"))
                    && (tc.ends_with("; end") || tc.ends_with(";end"));
                if is_rescue_end {
                    stack.pop();
                    continue;
                }
            }

            if t.starts_with("def ") || t == "def" {
                if is_endless_method(t) {
                    // Endless methods (`def foo = expr`) have no `end` token;
                    // do not push a frame.
                    continue;
                }

                // Single-line `def method; body; end` — the `end` is on the same
                // line as the `def`. The line-by-line scan would never see the
                // closing `end`, leaving a frame permanently on the stack. Skip
                // frame changes entirely for these one-liners.
                if is_one_liner_def(t) {
                    continue;
                }

                // Flag only when directly inside another def (top frame is Def).
                if stack.last() == Some(&FrameKind::Def) {
                    // Nested def — flag it (unless it's a singleton method def `def obj.method`)
                    // Skip singleton method definitions like `def self.foo` or `def obj.foo`
                    let after_def = t.strip_prefix("def ").unwrap_or("");
                    let is_singleton = after_def.contains('.') && !after_def.starts_with('(');

                    if !is_singleton {
                        let indent = raw.len() - trimmed.len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Method defined inside another method.".into(),
                            range: TextRange::new(pos, pos + t.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }
                stack.push(FrameKind::Def);
            } else if t == "end"
                || (t.starts_with("end") && matches!(
                    t.as_bytes().get(3).copied(),
                    Some(c) if !c.is_ascii_alphanumeric() && c != b'_' && c != b':'
                ))
            {
                // Pop the top frame (if any). Do not push anything.
                // The word-boundary check (not alphanumeric, not _, not :) prevents
                // `endless` / `end:` from being treated as `end` tokens.
                stack.pop();
            } else if opens_other_block(t) && !is_one_liner_block(t) {
                stack.push(FrameKind::Other);
            }

            // Brace-block tracking for anonymous class/module/struct definitions:
            // `Class.new { ... }`, `Module.new { ... }`, `Struct.new { ... }`.
            // These open a new class scope — `def` inside is NOT a nested method.
            // We only target these specific patterns to avoid spurious pushes for
            // hash literals and other uses of `{` that span multiple lines.
            let bare_for_brace = strip_trailing_comment(t).trim_end();
            let is_anon_class_brace = (bare_for_brace.contains("Class.new")
                || bare_for_brace.contains("Module.new")
                || bare_for_brace.contains("Struct.new"))
                && bare_for_brace.contains('{')
                && {
                    let open_count = bare_for_brace.chars().filter(|&c| c == '{').count();
                    let close_count = bare_for_brace.chars().filter(|&c| c == '}').count();
                    open_count > close_count
                };
            if is_anon_class_brace {
                stack.push(FrameKind::Other);
            } else if t.starts_with('}') && !stack.is_empty() {
                // Closing brace: pop the matching anonymous-class frame.
                // Only pop if the top frame is Other (conservative: don't pop Def frames).
                if stack.last() == Some(&FrameKind::Other) {
                    stack.pop();
                }
            }
        }

        diags
    }
}

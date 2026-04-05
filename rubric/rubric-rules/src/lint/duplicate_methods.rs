use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashMap;

pub struct DuplicateMethods;

/// Count word-boundary `end` tokens on the line, skipping string literal
/// contents and stopping at inline `#` comments.
///
/// Without this, `end` as a word inside a string (e.g. `"unexpected end of
/// input"`, `"at the end of the day"`) or after a `#` (e.g. `end # end here`)
/// is miscounted, corrupting the frame stack and causing false positives.
fn count_ends(line: &str) -> i64 {
    if line.trim_start().starts_with('#') {
        return 0;
    }
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut count = 0i64;
    let mut i = 0usize;
    let mut in_string: Option<u8> = None; // Some(b'"') or Some(b'\'')

    while i < n {
        let b = bytes[i];
        match in_string {
            Some(q) => {
                if b == b'\\' && i + 1 < n {
                    // Skip escaped character (handles \', \", \\, etc.)
                    i += 2;
                } else if b == q {
                    in_string = None;
                    i += 1;
                } else {
                    i += 1;
                }
            }
            None => {
                if b == b'#' {
                    // Rest of line is an inline comment — stop counting.
                    break;
                }
                if b == b'"' || b == b'\'' {
                    in_string = Some(b);
                    i += 1;
                    continue;
                }
                if i + 2 < n && &bytes[i..i + 3] == b"end" {
                    let before_ok = i == 0
                        || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
                    let after_ok = i + 3 >= n
                        || (!bytes[i + 3].is_ascii_alphanumeric() && bytes[i + 3] != b'_');
                    if before_ok && after_ok {
                        count += 1;
                    }
                    i += 3;
                } else {
                    i += 1;
                }
            }
        }
    }
    count
}

/// Returns `true` if the line has ` do` immediately before an inline comment,
/// e.g. `ActiveSupport.on_load(:action_controller) do # :nodoc:`.
/// This catches do-block openers that `t.ends_with(" do")` misses.
fn has_do_before_comment(t: &str) -> bool {
    // Look for ` do ` where the rest (after trimming) is a `#` comment.
    if let Some(pos) = t.find(" do ") {
        let after = t[pos + 4..].trim_start();
        if after.is_empty() || after.starts_with('#') {
            return true;
        }
    }
    false
}

/// Classify a fully-trimmed line.
/// Returns `(opens_block, isolates_method_namespace)`.
///
/// Only these keywords open a block that requires a matching `end`:
///   class, module, class<<self  → isolating (own method namespace)
///   do-blocks                   → isolating (RSpec DSL, Class.new, etc.)
///   def                         → non-isolating
///   if, unless, while, until, for, case, begin → non-isolating
///
/// NOTE: `rescue`, `else`, `elsif`, `ensure` are clause separators inside
/// existing blocks — they do NOT open a new block and do NOT consume an `end`.
fn classify_opener(t: &str) -> (bool, bool) {
    // class/module/class<<self — new isolating scope
    if t.starts_with("class ") || t == "class"
        || t.starts_with("module ") || t == "module"
        || t.starts_with("class << ")
    {
        return (true, true);
    }

    // `do` block — isolating (RSpec, Thread.new, Class.new, etc.)
    // Also handle trailing inline comments: `foo do # :nodoc:` or `foo do # comment`.
    if t == "do"
        || t.ends_with(" do")
        || t.contains(" do |")
        || t.contains(" do\t")
        || has_do_before_comment(t)
    {
        return (true, true);
    }

    // def — opens a block but NOT a new method namespace
    if t.starts_with("def ") || t == "def" {
        return (true, false);
    }

    // Control-flow block openers (each needs a matching `end`)
    for kw in &["if ", "unless ", "while ", "until ", "for ", "case ", "begin"] {
        if t.starts_with(kw) || t == *kw {
            return (true, false);
        }
    }

    // Inline `begin` at end of line — e.g. `@cache ||= begin` or `result = begin`.
    // These open a `begin...end` block that requires a matching `end`.
    if t.ends_with(" begin") {
        return (true, false);
    }

    // Inline assignment block openers: `foo = if cond`, `@x ||= case val`, etc.
    // The keyword immediately follows an assignment operator (`=`, `||=`, `&&=`, etc.)
    // and starts a block that requires a matching `end`.
    if has_assignment_block_opener(t) {
        return (true, false);
    }

    (false, false)
}

/// Returns `true` when the line contains an assignment followed immediately by
/// a block-opening keyword (`if`, `unless`, `case`, `begin`), e.g.:
///   `@x ||= if condition`
///   `named_route = case value`
///
/// These are NOT detected by `starts_with` checks because the keyword is
/// not at the beginning of the line, yet they DO require a matching `end`.
fn has_assignment_block_opener(t: &str) -> bool {
    for kw in &[" if ", " unless ", " case ", " begin "] {
        // For " begin " we also check the end-of-line form separately,
        // so here we only match the mid-line form (" begin ").
        if let Some(kw_pos) = t.find(kw) {
            let before = t[..kw_pos].trim_end();
            if before.ends_with('=')
                && !before.ends_with("==")
                && !before.ends_with("!=")
                && !before.ends_with("<=")
                && !before.ends_with(">=")
                && !before.ends_with("=>")
            {
                return true;
            }
        }
    }
    false
}

/// Try to extract a heredoc marker from the line.
/// Returns `Some(marker)` if the line opens a heredoc (<<MARKER or <<~MARKER or <<"MARKER" etc.).
/// The returned marker is what to look for as the closing line (stripped of quotes).
fn heredoc_marker(line: &str) -> Option<String> {
    // Look for << followed by optional ~ and optional quotes, then the marker identifier.
    let mut rest = line;
    loop {
        let pos = rest.find("<<")?;
        let after = &rest[pos + 2..];
        // Skip optional ~ or -
        let after = after.strip_prefix('~').or_else(|| after.strip_prefix('-')).unwrap_or(after);
        // Skip optional quote char
        let (after, _quoted) = if after.starts_with('"') || after.starts_with('\'') || after.starts_with('`') {
            (&after[1..], true)
        } else {
            (after, false)
        };
        // Collect the identifier characters
        let end = after
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after.len());
        if end > 0 {
            return Some(after[..end].to_string());
        }
        // Try further in the line
        if pos + 2 < rest.len() {
            rest = &rest[pos + 2..];
        } else {
            break;
        }
    }
    None
}

impl Rule for DuplicateMethods {
    fn name(&self) -> &'static str {
        "Lint/DuplicateMethods"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Each stack frame: (seen_methods, depth, isolates_namespace)
        // seen_methods maps method_name → (first_line, is_conditional).
        // is_conditional = true when the method was defined inside a conditional frame
        // (if/unless/while/begin etc.) rather than directly in the isolating scope.
        // Two conditional definitions never flag each other — they may be in mutually
        // exclusive branches (if/else, begin/rescue).
        // Bottom frame: the file scope. depth = -1 (sentinel: never popped). isolates = true.
        let mut stack: Vec<(HashMap<String, (usize, bool)>, i64, bool)> =
            vec![(HashMap::new(), -1i64, true)];

        // Heredoc state: if Some(marker), we're inside a heredoc body and skip until marker.
        let mut heredoc_end: Option<String> = None;

        for i in 0..n {
            let raw = &lines[i];
            let trimmed = raw.trim_start();
            let t = trimmed.trim();

            // --- Heredoc body: skip until closing marker ---
            if let Some(ref marker) = heredoc_end.clone() {
                if t == marker.as_str() {
                    heredoc_end = None;
                }
                continue;
            }

            if t.is_empty() || t.starts_with('#') {
                continue;
            }

            // --- Detect heredoc opening on this line ---
            // If this line opens a heredoc, subsequent lines until the marker are body.
            let opens_heredoc = heredoc_marker(t);

            let mut remaining_ends = count_ends(t);
            let (opens, isolates) = classify_opener(t);

            // --- Push new frame for block-openers ---
            if opens {
                stack.push((HashMap::new(), 1i64, isolates));
            }


            // --- Brace-block isolating scope: Class.new { }, Module.new { }, Struct.new { } ---
            // These open a new class scope using `{...}` (not `do...end`), so they are
            // not detected by `classify_opener`. We push an isolating frame with depth = -2
            // (a sentinel distinct from -1 = file scope) that is skipped by the `end` loop
            // and closed when `}` appears at the start of a line.
            const BRACE_FRAME_DEPTH: i64 = -2;
            let brace_text = t;
            let opens_brace_scope = (brace_text.contains("Class.new")
                || brace_text.contains("Module.new")
                || brace_text.contains("Struct.new"))
                && brace_text.contains('{')
                && {
                    let open_count = brace_text.chars().filter(|&c| c == '{').count();
                    let close_count = brace_text.chars().filter(|&c| c == '}').count();
                    open_count > close_count
                };
            if opens_brace_scope {
                stack.push((HashMap::new(), BRACE_FRAME_DEPTH, true));
            }

            // Close brace scope when `}` appears at the start of the trimmed line.
            if t.starts_with('}') && stack.last().map(|f| f.1) == Some(BRACE_FRAME_DEPTH) {
                stack.pop();
            }

            // --- `undef` removes a method from the scope ---
            // Pattern: `undef items_for` before redefining — not a duplicate.
            if trimmed.starts_with("undef ") {
                let name = trimmed["undef ".len()..].trim();
                // Remove from the nearest isolating frame so the next `def` isn't flagged.
                let len = stack.len();
                if let Some(idx) = (0..len).rev().find(|&i| stack[i].2) {
                    stack[idx].0.remove(name);
                }
            }

            // --- Record/check `def` method name ---
            if trimmed.starts_with("def ") {
                let after_def = &trimmed["def ".len()..];
                let name_end = after_def
                    .find(|c: char| c == '(' || c == ' ' || c == '\n' || c == ';')
                    .unwrap_or(after_def.len());
                let method_name = after_def[..name_end].trim();

                if !method_name.is_empty() {
                    // Skip singleton method definitions on variable receivers:
                    // `def some_var.method_name` — the method is defined on a specific object,
                    // not on the class, and the receiver identifies which object it's on.
                    // Only `def self.method_name` should be tracked (it defines a class method
                    // that genuinely conflicts if repeated).
                    let is_singleton_on_var = if let Some(dot_pos) = method_name.find('.') {
                        &method_name[..dot_pos] != "self"
                    } else {
                        false
                    };

                    if !is_singleton_on_var {
                        // Walk down to find the nearest isolating frame (above the def frame).
                        let len = stack.len();
                        let isolating_idx = (0..len.saturating_sub(1))
                            .rev()
                            .find(|&idx| stack[idx].2);

                        if let Some(idx) = isolating_idx {
                            // A method is "conditional" when there are intermediate non-isolating
                            // frames between the isolating scope and the def frame.  Such defs
                            // may live in mutually exclusive branches (if/else, begin/rescue),
                            // so two conditional defs with the same name are NOT flagged.
                            let is_conditional = len.saturating_sub(2) > idx;

                            if let Some(&(first_line, first_is_conditional)) =
                                stack[idx].0.get(method_name)
                            {
                                // Only flag when the FIRST definition was unconditional.
                                // If both are conditional they might be in separate if/else branches.
                                if !first_is_conditional {
                                    let indent = lines[i].len() - trimmed.len();
                                    let line_start = ctx.line_start_offsets[i] as usize;
                                    let pos = (line_start + indent) as u32;
                                    diags.push(Diagnostic {
                                        rule: self.name(),
                                        message: format!(
                                            "Duplicate method `{}` (first defined at line {}).",
                                            method_name,
                                            first_line + 1
                                        ),
                                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                                        severity: Severity::Warning,
                                    });
                                }
                            } else {
                                stack[idx].0.insert(method_name.to_string(), (i, is_conditional));
                            }
                        }
                    }
                }
            }

            // --- Apply `end` tokens, cascading through frames ---
            while remaining_ends > 0 {
                let top_depth = stack.last().unwrap().1;
                if top_depth < 0 {
                    // File-scope sentinel — never pop.
                    break;
                }
                let absorbed = remaining_ends.min(top_depth);
                stack.last_mut().unwrap().1 -= absorbed;
                remaining_ends -= absorbed;

                let d = stack.last().unwrap().1;
                if d <= 0 && stack.len() > 1 {
                    stack.pop();
                }
            }

            // --- After processing this line, set heredoc state if opened ---
            if let Some(marker) = opens_heredoc {
                heredoc_end = Some(marker);
            }
        }

        diags
    }
}

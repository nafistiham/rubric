use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ElseAlignment;

/// A stack entry tracking the kind of block opener seen.
#[derive(Debug)]
enum StackEntry {
    /// An `if`/`unless` block whose `else`/`elsif` we want to check.
    ///
    /// `primary` is the column of the `if`/`unless` keyword, used as the
    /// expected indent for `else`/`elsif`.
    ///
    /// `alt` carries the line's leading-whitespace indent when `if`/`unless`
    /// appears in an assignment context (`x = if cond`, `x ||= unless cond`).
    /// In that pattern both the keyword column AND the line base indent are
    /// accepted as valid alignment for `else`/`elsif`.
    If { primary: usize, alt: Option<usize> },
    /// A `case` block — its `when`/`else` belongs to `case`, not to `if`.
    /// We do NOT flag `else` inside a `case` block.
    Case,
    /// Any other block opener (`def`, `class`, `module`, `while`, `until`,
    /// `begin`, `do`) — we track these only to correctly consume their `end`
    /// tokens so they don't mis-pop an `If` entry.
    Other,
}

/// Extracts the heredoc terminator word from a line, returning `None` if not a heredoc opener.
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

impl Rule for ElseAlignment {
    fn name(&self) -> &'static str {
        "Layout/ElseAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Unified stack of block openers. Each entry is If{..}, Case, or Other.
        // Every `end` pops exactly one entry, preserving correct interleaving.
        let mut stack: Vec<StackEntry> = Vec::new();

        // Heredoc tracking
        let mut in_heredoc: Option<String> = None;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            // Skip heredoc body lines.
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // Detect heredoc opener — body starts on next line.
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: opener line contains real Ruby.
            }

            if trimmed.starts_with('#') {
                continue;
            }

            let t = trimmed.trim();

            // ── Detect `if`/`unless` openers ─────────────────────────────────
            // When the keyword starts the expression (trimmed line starts with
            // `if `/`unless `), the `else`/`elsif` must align with the line's
            // leading indent — which equals the keyword column.
            if t.starts_with("if ") || t.starts_with("unless ") {
                stack.push(StackEntry::If { primary: indent, alt: None });
            }
            // Inline assignment `x = if cond` / `x ||= unless cond` — the
            // `else`/`elsif` may align with EITHER the `if`/`unless` keyword
            // column OR the line's base indent.  Both are accepted as valid.
            else if !t.starts_with("if ") && !t.starts_with("unless ") {
                // Check for `= if ` / `= unless ` patterns
                let inline_if_col = find_inline_keyword_col(t, "if");
                let inline_unless_col = find_inline_keyword_col(t, "unless");
                if let Some(col) = inline_if_col.or(inline_unless_col) {
                    // `primary` = column of the `if`/`unless` keyword in the
                    //   full line (used when `else` is aligned under the keyword).
                    // `alt`     = line's leading indent (used when `else` is
                    //   aligned under the start of the assignment).
                    let keyword_col = indent + col;
                    // Only record alt when keyword_col != indent (they differ
                    // in assignment context; equal when the keyword is the first
                    // non-whitespace token, which is handled by the branch above).
                    let alt = if keyword_col != indent { Some(indent) } else { None };
                    stack.push(StackEntry::If { primary: keyword_col, alt });
                }
                // ── Detect `case` openers (tracked separately from `if`) ──────
                else if t.starts_with("case ") || t == "case" {
                    stack.push(StackEntry::Case);
                }
                // Inline case: `x = case y` / `x =  case y` (any whitespace) — push Case
                // so its `else`/`when` is skipped (not an `if`/`unless` branch).
                else if contains_assign_kw(t, "case") {
                    stack.push(StackEntry::Case);
                }
                // ── Detect other non-if block openers ─────────────────────────
                else if t.starts_with("def ")
                    || t == "def"
                    || t == "begin"
                    || t.starts_with("begin ")
                    || t.ends_with(" begin")  // e.g. `@x ||= begin`
                    || t.starts_with("while ")
                    || t.starts_with("until ")
                    || t.starts_with("class ")
                    || t.starts_with("module ")
                    || t.ends_with(" do")
                    || t.contains(" do |")
                    || t.contains(" do|")
                    || t == "do"
                {
                    stack.push(StackEntry::Other);
                }
            }

            // ── Detect `else`/`elsif` ─────────────────────────────────────────
            if t == "else" || t.starts_with("elsif ") || t == "elsif" {
                // Check the top of the stack:
                // - If the topmost entry is Case, this `else` belongs to `case` → skip.
                // - If the topmost entry is If{primary,alt}, check alignment against both.
                // - If topmost is Other (a do/def/begin inside an if), walk down.
                if let Some((primary, alt)) = find_controlling_if_indent(&stack) {
                    let valid = indent == primary || alt.map_or(false, |a| indent == a);
                    if !valid {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`else`/`elsif` at indent {} should be at {}.",
                                indent, primary
                            ),
                            range: TextRange::new(pos, pos + trimmed.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            // ── Detect `end` — pops exactly one stack entry ───────────────────
            // `end` followed by any non-identifier char (end, end.x, end,, end), etc.)
            let is_end = t.starts_with("end")
                && (t.len() == 3 || {
                    let b = t.as_bytes()[3];
                    !b.is_ascii_alphanumeric() && b != b'_'
                });

            if is_end {
                stack.pop();
            }
        }

        diags
    }
}

/// Find the expected indent(s) for `else`/`elsif` given the current stack.
///
/// Returns `Some((primary, alt))` where:
/// - `primary` is the column of the `if`/`unless` keyword.
/// - `alt` is `Some(line_indent)` when the `if` was in an assignment context,
///   `None` otherwise.
///
/// Rules:
/// - Walk from the top of the stack.
/// - If we hit a `Case` entry first → return `None` (this `else` belongs to `case`, skip check).
/// - If we hit an `If{primary,alt}` entry first (possibly after some `Other` entries) → return it.
/// - If stack is empty or only has `Other`/`Case` with no `If` above → return `None`.
///
/// Walking past `Other` is correct because `do`/`def`/`begin` blocks can be nested INSIDE
/// an `if`, and the `else` then belongs to the outer `if`. E.g.:
/// ```ruby
/// if cond
///   items.each do |x|   # Other pushed
///   end                  # Other popped
/// else                   # else belongs to the if
/// ```
/// However, we must STOP at `Case` entries because `case`'s `else` belongs to `case`.
fn find_controlling_if_indent(stack: &[StackEntry]) -> Option<(usize, Option<usize>)> {
    for entry in stack.iter().rev() {
        match entry {
            StackEntry::If { primary, alt } => return Some((*primary, *alt)),
            StackEntry::Case => return None, // else belongs to case, not to an if
            StackEntry::Other => {
                // Keep walking — this Other is inside an if, and the else
                // may belong to the outer if.
                continue;
            }
        }
    }
    None
}

/// Find the column of an inline `if` or `unless` keyword after an assignment.
/// Pattern: `... = if ` / `... =  if ` (any whitespace) / `... ||= if ` etc.
/// Returns the byte offset within `t` (the trimmed line) of the keyword's first char,
/// or `None` if not found.
fn find_inline_keyword_col(t: &str, kw: &str) -> Option<usize> {
    let bytes = t.as_bytes();
    let n = bytes.len();
    let kw_bytes = kw.as_bytes();
    let kw_len = kw_bytes.len();

    let mut i = 0;
    while i < n {
        if bytes[i] == b'=' {
            // Skip `==`, `=>`, `=~`
            if i + 1 < n && (bytes[i + 1] == b'=' || bytes[i + 1] == b'>' || bytes[i + 1] == b'~') {
                i += 1;
                continue;
            }
            // Skip spaces after `=`
            let mut j = i + 1;
            while j < n && bytes[j] == b' ' { j += 1; }
            // Check for kw at position j
            if j + kw_len <= n && &bytes[j..j + kw_len] == kw_bytes {
                let after_kw = j + kw_len;
                if after_kw >= n || bytes[after_kw] == b' ' || bytes[after_kw] == b'\n' {
                    return Some(j);
                }
            }
        } else if i + 2 < n && bytes[i] == b'<' && bytes[i + 1] == b'<' {
            // `<< kw`
            let mut j = i + 2;
            while j < n && bytes[j] == b' ' { j += 1; }
            if j + kw_len <= n && &bytes[j..j + kw_len] == kw_bytes {
                let after_kw = j + kw_len;
                if after_kw >= n || bytes[after_kw] == b' ' {
                    return Some(j);
                }
            }
        }
        i += 1;
    }
    None
}

/// Returns true if `t` contains `= kw` (with any amount of whitespace between `=` and `kw`)
/// where `kw` is followed by a space or end-of-string.
/// Skips `==`, `=>`, `=~` to avoid false matches.
fn contains_assign_kw(t: &str, kw: &str) -> bool {
    find_assign_kw_pos(t, kw).is_some()
}

fn find_assign_kw_pos(t: &str, kw: &str) -> Option<usize> {
    let bytes = t.as_bytes();
    let n = bytes.len();
    let kw_bytes = kw.as_bytes();
    let kw_len = kw_bytes.len();
    let mut i = 0;
    while i < n {
        if bytes[i] == b'=' {
            if i + 1 < n && (bytes[i + 1] == b'=' || bytes[i + 1] == b'>' || bytes[i + 1] == b'~') {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < n && bytes[j] == b' ' { j += 1; }
            if j + kw_len <= n && &bytes[j..j + kw_len] == kw_bytes {
                let after = j + kw_len;
                if after >= n || bytes[after] == b' ' || bytes[after] == b'\n' {
                    return Some(j);
                }
            }
        }
        i += 1;
    }
    None
}

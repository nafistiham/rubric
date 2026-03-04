use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct BlockAlignment;

// Find the column where the "block receiver" starts on a `do` opener line.
// For `top_hashtags: top_hashtags.map do |(name, count)|` the block receiver
// is `top_hashtags.map` which starts at the position after `top_hashtags: `.
//
// Algorithm: find the ` do ` (or ` do|` or ` do\n`) token in the line, then
// scan backwards past the method chain (identifier, `.`, method names) to find
// the start of the leftmost component.  If no inner receiver is found (i.e.,
// the chain starts at the line's first non-space character), return None.
fn block_receiver_start_col(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    // Find ` do ` position in the raw line
    let do_pos = {
        let mut found = None;
        let len = bytes.len();
        let mut i = 0;
        while i + 2 < len {
            if bytes[i] == b' ' && bytes[i + 1] == b'd' && bytes[i + 2] == b'o' {
                let after = bytes.get(i + 3).copied().unwrap_or(b' ');
                if after == b' ' || after == b'|' || after == b'\n' || after == b'\r' || i + 3 == len {
                    found = Some(i);
                    break;
                }
            }
            i += 1;
        }
        found?
    };

    // Scan backwards from do_pos to find the start of the call chain
    // Skip spaces before `do`
    let mut pos = do_pos;
    while pos > 0 && bytes[pos - 1] == b' ' {
        pos -= 1;
    }

    // Now walk back through the chain: word chars, `.`, `(`, `)`, `[`, `]`
    // We want to find the leftmost word start that is after whitespace or `:` or `=`
    let end_of_chain = pos;
    while pos > 0 {
        let ch = bytes[pos - 1];
        if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'.' || ch == b'!' || ch == b'?' {
            pos -= 1;
        } else if ch == b')' || ch == b']' {
            // Skip parenthesized argument list
            let mut depth = 1i32;
            pos -= 1;
            while pos > 0 && depth > 0 {
                pos -= 1;
                match bytes[pos] {
                    b')' | b']' => depth += 1,
                    b'(' | b'[' => depth -= 1,
                    _ => {}
                }
            }
        } else {
            break;
        }
    }

    // `pos` is now the start of the chain
    let chain_start = pos;
    let line_indent = line.len() - line.trim_start().len();

    // Only return the column if it's DIFFERENT from the line indent (otherwise
    // it would be redundant — the line-indent check already covers that)
    if chain_start > line_indent && chain_start < end_of_chain {
        Some(chain_start)
    } else {
        None
    }
}

// Returns true if a trimmed line is an `end` statement.
// Recognises `end` followed by punctuation such as `)`, `}`, `]`, `,`, `;`
// in addition to the plain keyword and `end.method` / `end # comment` forms.
fn is_end_line(t: &str) -> bool {
    t == "end"
        || t.starts_with("end.")
        || t.starts_with("end ")
        || t.starts_with("end)")
        || t.starts_with("end}")
        || t.starts_with("end]")
        || t.starts_with("end,")
        || t.starts_with("end;")
}

// Returns true if the line ends with a bare `begin` keyword, meaning the line
// opens a `begin...end` block expression (e.g. `x = begin`, `x ||= begin`).
fn line_ends_with_begin(t: &str) -> bool {
    let code = if let Some(idx) = t.find(" #") { &t[..idx] } else { t };
    let code = code.trim_end();
    code == "begin" || code.ends_with(" begin") || code.ends_with("\tbegin")
}

// Returns true if the line has `case` as the RHS of an assignment.
// Handles any amount of whitespace between `=` and `case`, e.g.:
//   `c.name = case foo`
//   `c.service_name =  case $PROGRAM_NAME`
fn line_has_rhs_case(t: &str) -> bool {
    let code = if let Some(idx) = t.find(" #") { &t[..idx] } else { t };
    let code = code.trim_end();
    // Ends with just `case` after whitespace (entire RHS is `case` with expr on next lines)
    if code.ends_with(" case") || code.ends_with("\tcase") {
        return true;
    }
    // Contains `= case ` or `= case\t` with any amount of whitespace around `case`
    // Pattern: `=` then optional spaces then `case` then space/tab/end
    let bytes = code.as_bytes();
    let mut i = 0;
    while i + 4 < bytes.len() {
        if bytes[i] == b'=' {
            // Check it's not `==` or `=>`
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = bytes.get(i + 1).copied().unwrap_or(0);
            if next != b'=' && next != b'>' && prev != b'!' && prev != b'=' && prev != b'<' && prev != b'>' {
                // Skip whitespace after `=`
                let mut j = i + 1;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                // Check if `case` follows
                if bytes.get(j..j + 4) == Some(b"case") {
                    let after = bytes.get(j + 4).copied().unwrap_or(b' ');
                    if after == b' ' || after == b'\t' || j + 4 == bytes.len() {
                        return true;
                    }
                }
            }
        }
        i += 1;
    }
    false
}

// Returns true if a `def` line will NOT produce a separate matching `end` on
// its own line.  This covers two cases:
//   1. Endless method:  `def foo = expr`
//   2. Single-line def: `def foo; body; end` or `def foo(x); end`
// Both must NOT be pushed as inner constructs because their `end` (if any) is
// inline and will not appear as a standalone `end` line.
fn is_endless_method_def(t: &str) -> bool {
    if !t.starts_with("def ") {
        return false;
    }

    // Single-line def: the line contains a semicolon outside of parens/strings
    // (e.g. `def foo; body; end` or `def foo(x); end`).  These already carry
    // their `end` inline, so they must not be pushed as inner constructs.
    {
        let mut depth: i32 = 0;
        let mut in_str: Option<u8> = None;
        for &b in t.as_bytes() {
            match in_str {
                Some(d) if b == d => { in_str = None; }
                Some(_) => {}
                None if b == b'\'' || b == b'"' => { in_str = Some(b); }
                None if b == b'(' => { depth += 1; }
                None if b == b')' => { depth -= 1; }
                None if b == b';' && depth == 0 => { return true; }
                None => {}
            }
        }
    }

    let rest = &t[4..];
    let mut depth: i32 = 0;
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => { depth -= 1; }
            b'=' if depth == 0 => {
                let next = bytes.get(i + 1).copied().unwrap_or(0);
                let prev = if i > 0 { bytes[i - 1] } else { 0 };
                // Exclude compound/comparison operators: ==, =>, !=, <=, >=, +=, -=, etc.
                if next == b'=' || next == b'>' {
                    i += 1;
                    continue;
                }
                if prev == b'!' || prev == b'<' || prev == b'>'
                    || prev == b'=' || prev == b'+'
                    || prev == b'-' || prev == b'*'
                    || prev == b'/' || prev == b'%'
                    || prev == b'|' || prev == b'&'
                {
                    i += 1;
                    continue;
                }
                // Exclude setter method names like `def foo=(x)` where `=` immediately
                // follows a word character (letter, digit, underscore, `?`, `!`).
                // Endless def `=` must be preceded by space, tab, or `)`.
                if prev != b' ' && prev != b'\t' && prev != b')' {
                    i += 1;
                    continue;
                }
                // `=` preceded by space or `)` and not part of a compound op: endless def
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

impl Rule for BlockAlignment {
    fn name(&self) -> &'static str {
        "Layout/BlockAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Stack entries: (chain_indent, do_line_indent, opener_line_idx, is_do_block)
        //   is_do_block = true  -> do...end block
        //     `end` must align with chain_indent, do_line_indent, or any word-boundary column
        //     between chain_indent and the `do` keyword on the opener line.
        //   is_do_block = false -> inner construct (if/def/begin/case/…): just balanced, no diagnostic
        //
        // For non-chain openers, chain_indent == do_line_indent.
        // For chain openers (trimmed starts with `.`), chain_indent is the first line of the chain.
        let mut stack: Vec<(usize, usize, usize, bool)> = Vec::new();

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            if trimmed.starts_with('#') {
                continue;
            }

            let t = trimmed.trim();

            // ── What does this line open? ─────────────────────────────────

            let opens_do_block = t.ends_with(" do")
                || t.ends_with(" do |")
                || t.contains(" do |")
                || t.contains(" do|")
                || t == "do";

            // Inner constructs that each require one matching `end`, but whose
            // alignment relative to the `end` is NOT checked here.
            let opens_inner_construct = !opens_do_block && (
                // Bare `begin` statement or `begin` as RHS of assignment / compound-assign
                t == "begin"
                    || t.starts_with("begin ")
                    || line_ends_with_begin(t)
                    || line_has_rhs_case(t)
                    // `case foo` / `case` as a standalone statement
                    || t.starts_with("case ")
                    || t == "case"
                    // Regular (non-endless) method definition
                    || (!is_endless_method_def(t) && (t.starts_with("def ") || t == "def"))
                    || t.starts_with("if ")
                    || t.starts_with("unless ")
                    || t.starts_with("while ")
                    || t.starts_with("until ")
                    || t.starts_with("class ")
                    || t.starts_with("module ")
            );

            // Inline conditional on a shovel/assignment that is not a do-block or inner construct.
            // Patterns: `x << if`, `x = if`, `x += if`, `x ||= if`, etc.
            let has_inline_conditional = !opens_do_block && !opens_inner_construct && (
                t.contains(" << if ")
                    || t.contains(" << unless ")
                    || t.contains(" << case ")
                    || t.contains(" = if ")
                    || t.contains(" = unless ")
                    || t.contains("= if ")    // catches `+=`, `||=`, `&&=`, etc.
                    || t.contains("= unless ")
            );

            if opens_do_block {
                // For method-chain openers the `do` keyword may be on a continuation
                // line indented deeper than the logical start of the expression.
                // Rubocop allows `end` to align with either the chain's first line OR
                // the line containing `do`, so we record both.
                //
                // Two continuation patterns handled:
                //   1. Dot-chain: trimmed starts with `.` (e.g., `.in_batches do`)
                //   2. Multi-line args: trimmed line ends with `) do` or `] do` — the `)`
                //      closes a multiline call whose opening line is at a lower indent.
                let (chain_indent, do_line_indent) = {
                    let raw_line = lines[i];
                    // Pattern 1: dot-chain continuation
                    if t.starts_with('.') {
                        let mut ci = indent;
                        let mut j = i;
                        while j > 0 {
                            j -= 1;
                            let prev = &lines[j];
                            let prev_trimmed = prev.trim_start();
                            if prev_trimmed.starts_with('#') {
                                continue;
                            }
                            let prev_indent = prev.len() - prev_trimmed.len();
                            ci = prev_indent;
                            if !prev_trimmed.is_empty() && !prev_trimmed.starts_with('.') {
                                break;
                            }
                        }
                        (ci, indent)
                    }
                    // Pattern 2: the `do` is preceded by `)` or `]` on the same line,
                    // meaning the block is passed to a call whose args span multiple lines.
                    // Scan backward to find the line with the matching opening `(` or `[`.
                    else if {
                        // Is there a `)` or `]` before ` do` on this line?
                        let b = raw_line.as_bytes();
                        let has_close_before_do = if let Some(do_off) = raw_line.find(" do") {
                            b[..do_off].iter().rev().any(|&c| c == b')' || c == b']')
                        } else {
                            false
                        };
                        has_close_before_do
                    } {
                        // Scan backward to find the line whose paren depth first goes negative
                        // (i.e., where the unmatched `(` lives).
                        let mut depth: i32 = 0;
                        let mut ci = indent;
                        let mut j = i;
                        // Count parens in opener line itself
                        for &b in raw_line.as_bytes() {
                            match b {
                                b'(' | b'[' => depth -= 1,
                                b')' | b']' => depth += 1,
                                _ => {}
                            }
                        }
                        // Walk backward until we balance the parens
                        while j > 0 && depth > 0 {
                            j -= 1;
                            let prev = &lines[j];
                            let prev_trimmed = prev.trim_start();
                            if prev_trimmed.starts_with('#') {
                                continue;
                            }
                            for &b in prev.as_bytes() {
                                match b {
                                    b'(' | b'[' => depth -= 1,
                                    b')' | b']' => depth += 1,
                                    _ => {}
                                }
                            }
                            if depth <= 0 {
                                ci = prev.len() - prev_trimmed.len();
                                break;
                            }
                        }
                        if ci < indent {
                            (ci, indent)
                        } else {
                            (indent, indent)
                        }
                    }
                    else {
                        (indent, indent)
                    }
                };
                stack.push((chain_indent, do_line_indent, i, true));
            } else if opens_inner_construct || has_inline_conditional {
                // Only track inner constructs when we are already inside a do-block.
                if !stack.is_empty() {
                    stack.push((indent, indent, i, false));
                }
            }

            // ── Does this line close a block? ─────────────────────────────

            if is_end_line(t) {
                if let Some((chain_indent, do_line_indent, opener_idx, is_do)) = stack.pop() {
                    if is_do && indent != chain_indent && indent != do_line_indent {
                        // Additional check: `end` may align with the block's receiver start
                        // (the leftmost component of the method chain that precedes `do`),
                        // when that start differs from the line indent.  This handles cases
                        // like `top_hashtags: top_hashtags.map do … end,` where the `end`
                        // aligns with `top_hashtags.map` rather than the hash key.
                        let opener_line = &lines[opener_idx];
                        let receiver_col = block_receiver_start_col(opener_line);
                        let ok = receiver_col == Some(indent);
                        if !ok {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            let pos = (line_start + indent) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "`end` at indent {} does not match block start at indent {}.",
                                    indent, chain_indent
                                ),
                                range: TextRange::new(pos, pos + 3),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    // If !is_do: inner construct balanced, no diagnostic emitted.
                }
            }
        }

        diags
    }
}

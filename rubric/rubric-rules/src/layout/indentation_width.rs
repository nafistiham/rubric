use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IndentationWidth;

/// Strip a trailing inline comment from a line so that checks like
/// `ends_with(',')` work even when the comma is followed by `# comment`.
/// This is intentionally simplified: it finds the first bare `#` (not
/// inside a string) and drops everything from there.
fn strip_inline_comment(s: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => in_single = !in_single,
            b'"'  if !in_single => in_double = !in_double,
            b'#'  if !in_single && !in_double => return &s[..i],
            b'\\' if in_single || in_double => i += 1, // skip escaped char
            _ => {}
        }
        i += 1;
    }
    s
}

/// Return true when `line` opens a heredoc — i.e. contains `<<~`, `<<-`, or
/// `<<` followed by an identifier/quoted delimiter.
fn opens_heredoc(line: &str) -> bool {
    line.contains("<<~") || line.contains("<<-") || {
        // bare `<<IDENT` — check that `<<` is present and the next char after
        // `<<` is a letter/underscore or quote (not a space or operator char).
        if let Some(pos) = line.find("<<") {
            let after = &line[pos + 2..];
            after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '"' || c == '\'')
        } else {
            false
        }
    }
}

impl Rule for IndentationWidth {
    fn name(&self) -> &'static str {
        "Layout/IndentationWidth"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut prev_nonempty_line: &str = "";

        // Compute per-file base indent: the indentation of the first non-blank,
        // non-comment code line. Files where everything is shifted by N spaces
        // (e.g. a global 1-space offset from a paste artifact) should be checked
        // relative to that base, not against absolute-zero alignment.
        // Rubocop checks delta-per-level, so a file with consistent 1-space base
        // is fine as long as each nesting level adds 2 more spaces.
        let base_indent: usize = {
            let mut base = 0usize;
            for line in ctx.lines.iter() {
                let t = line.trim();
                if t.is_empty() || t.starts_with('#') {
                    continue;
                }
                base = line.len() - line.trim_start_matches(' ').len();
                break;
            }
            base
        };

        // Track when we're inside an inline conditional (x = if / x = unless / x = case)
        // whose continuation lines have alignment-based indentation.
        let mut inline_cond_depth: usize = 0;
        // Track unclosed bracket depth across lines. When > 0 we are inside a bracket
        // expression and continuation lines use alignment indentation — skip odd-indent check.
        let mut bracket_depth: i32 = 0;
        // When a line with odd indentation is accepted (not flagged) because its previous
        // line ends with a continuation character, record its space count so that
        // subsequent lines at the same or deeper indent are also accepted (they are in
        // the same continuation block, e.g. do-block body or multiline string content).
        // Reset whenever indentation drops back below this level.
        let mut accepted_odd_indent: Option<usize> = None;
        // Heredoc tracking: when inside a heredoc body, skip all lines until the
        // closing delimiter.
        let mut in_heredoc = false;
        let mut heredoc_terminator: String = String::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }

            // ── Heredoc body detection ────────────────────────────────────
            // If we're currently inside a heredoc, skip the line unless it is
            // the terminator.
            if in_heredoc {
                let trimmed = line.trim();
                if trimmed == heredoc_terminator.as_str() {
                    in_heredoc = false;
                    heredoc_terminator.clear();
                }
                // Either way, don't check indentation of heredoc content.
                prev_nonempty_line = line;
                continue;
            }

            if line.starts_with('\t') {
                let start = ctx.line_start_offsets[i];
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use spaces, not tabs, for indentation.".into(),
                    range: TextRange::new(start, start + 1),
                    severity: Severity::Warning,
                });
                prev_nonempty_line = line;
                continue;
            }

            // ── Check whether this line opens a heredoc ───────────────────
            // We detect the terminator so we know when to exit heredoc mode.
            // Support: <<~TERM, <<-TERM, <<~'TERM', <<-'TERM', <<TERM.
            // We enter heredoc mode AFTER processing this line (the opener
            // line is normal Ruby and should be checked normally).
            let heredoc_opener_on_this_line = opens_heredoc(line);
            let new_heredoc_terminator: Option<String> = if heredoc_opener_on_this_line {
                // Extract the delimiter identifier from the heredoc marker.
                let marker = if let Some(p) = line.find("<<~") {
                    Some(&line[p + 3..])
                } else if let Some(p) = line.find("<<-") {
                    Some(&line[p + 3..])
                } else if let Some(p) = line.find("<<") {
                    Some(&line[p + 2..])
                } else {
                    None
                };
                marker.and_then(|rest| {
                    let rest = rest.trim_start();
                    // Quoted heredoc: `<<-'end;'` or `<<~"TERM"` — collect until closing quote
                    if let Some(q) = rest.chars().next().filter(|&c| c == '\'' || c == '"' || c == '`') {
                        let inner = &rest[q.len_utf8()..];
                        let term: String = inner.chars().take_while(|&c| c != q).collect();
                        if term.is_empty() { None } else { Some(term) }
                    } else {
                        // Unquoted: collect alphanumeric/underscore only
                        let term: String = rest
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        if term.is_empty() { None } else { Some(term) }
                    }
                })
            } else {
                None
            };

            // Capture the bracket depth at the START of this line (before updating it from
            // this line's content). Lines inside a bracket expression use alignment
            // indentation and must not be flagged for odd indent counts.
            let entering_depth = bracket_depth;

            // Update bracket depth from this line's content (simplified scan — ignores
            // string literals, but string-literal brackets don't typically cause alignment
            // indentation so the approximation is acceptable).
            for &b in line.as_bytes() {
                match b {
                    b'[' | b'(' | b'{' => bracket_depth += 1,
                    b']' | b')' | b'}' => {
                        bracket_depth -= 1;
                        if bracket_depth < 0 {
                            bracket_depth = 0;
                        }
                    }
                    b'#' => break, // stop at start of comment
                    _ => {}
                }
            }

            // Update inline conditional depth tracking.
            // Handle both `= if` and `||= if` / `&&= if` style assignments.
            {
                let l = line;
                if l.contains(" = if ")   || l.contains("||= if ")   || l.contains("&&= if ")
                || l.contains(" = unless ") || l.contains("||= unless ") || l.contains("&&= unless ")
                || l.contains(" = case ")   || l.contains("||= case ")   || l.contains("&&= case ")
                || l.contains(" << if ")   || l.contains(" << unless ") || l.contains(" << case ")
                || l.contains("!! case ") || l.contains("! case ")  // boolean-coerced case
                || l.contains(" || if ") || l.contains(" || unless ") || l.contains(" || case ")
                || l.contains(" && if ") || l.contains(" && unless ") || l.contains(" && case ")
                {
                    inline_cond_depth += 1;
                }
            }

            let spaces = line.len() - line.trim_start_matches(' ').len();
            // Check indentation relative to the file's base indent offset.
            // A file with a global 1-space offset still has valid 2-space-delta
            // indentation; we only flag when (spaces - base_indent) is odd.
            let effective_spaces = spaces.saturating_sub(base_indent);

            // Reset accepted_odd_indent when we return to a lower indent level.
            if let Some(ai) = accepted_odd_indent {
                if spaces < ai {
                    accepted_odd_indent = None;
                }
            }

            if spaces > 0 && effective_spaces % 2 != 0 {
                let prev_trim = prev_nonempty_line.trim_end();

                // Strip trailing inline comment from the previous line before
                // checking its ending character so a trailing `# comment` after
                // a comma does not defeat the comma-continuation heuristic.
                let prev_code = strip_inline_comment(prev_trim).trim_end();

                // Skip continuation lines — trailing comma means aligned argument continuation.
                // Also skip lines inside inline conditional expressions (alignment to `if` keyword).
                let is_comma_continuation = prev_code.ends_with(',');

                // Skip when previous line ends with an opening bracket/delimiter —
                // the next line uses alignment indentation to match the bracket position.
                let is_bracket_continuation = prev_code.ends_with('[')
                    || prev_code.ends_with('(')
                    || prev_code.ends_with('{')
                    || prev_code.ends_with('|');  // block params: `do |x|` / `{|x|`

                // Skip when previous line ends with a backslash line continuation.
                let is_backslash_continuation = prev_trim.ends_with('\\');

                // Skip when previous line ends with a boolean or arithmetic operator —
                // the current line is a continuation of the expression and may be
                // aligned to the operand on the first line.
                let is_boolean_continuation = prev_code.ends_with("&&")
                    || prev_code.ends_with("||")
                    || prev_code.ends_with("||=")
                    || prev_code.ends_with("&&=")
                    || prev_code.ends_with(" and")
                    || prev_code.ends_with(" or")
                    || prev_code.ends_with(" not");
                // Skip when previous line ends with an arithmetic/string operator that
                // signals a multi-line expression continuation (alignment to operand).
                let is_operator_continuation = prev_code.ends_with(" +")
                    || prev_code.ends_with(" -")
                    || prev_code.ends_with(" *")
                    || prev_code.ends_with(" /")
                    || prev_code.ends_with(" %")
                    || prev_code.ends_with(" <<")
                    || prev_code.ends_with(" >>");

                // Skip when previous line ends with `.` — method chain split where
                // continuation line starts with the next method name (not a dot).
                let is_dot_chain_continuation = prev_code.ends_with('.');

                // Skip when previous line ends with `?` or ` :` — ternary continuation.
                let is_ternary_continuation = prev_code.ends_with('?')
                    || prev_code.ends_with(" :");

                // Skip when previous line ends with `begin` — continuation body aligned
                // to the `begin` keyword (e.g. `params = begin` then `  body` aligned).
                let is_begin_continuation = prev_code.ends_with(" begin")
                    || prev_code == "begin";

                // Skip when current line starts with a method-chain dot — this line
                // is aligned to the receiver on the previous line.
                let trimmed_line = line.trim_start();
                let is_method_chain = trimmed_line.starts_with('.');

                // Skip lines immediately following branch keywords (else/elsif/rescue/ensure/when).
                // These lines are indented relative to the branch keyword's alignment position
                // (which may itself be at an odd indent inside an inline conditional).
                let prev_trimmed_start = prev_nonempty_line.trim_start();
                let is_after_branch_keyword = prev_trimmed_start.starts_with("else")
                    || prev_trimmed_start.starts_with("elsif ")
                    || prev_trimmed_start.starts_with("rescue")
                    || prev_trimmed_start.starts_with("ensure")
                    || prev_trimmed_start.starts_with("when ");

                // Skip closing tokens — they align with their opener, which may be odd-indented.
                let is_end_keyword = trimmed_line == "end"
                    || trimmed_line.starts_with("end ")
                    || trimmed_line.starts_with("end.")
                    || trimmed_line.starts_with("end\n");
                // Skip branch keywords: elsif/else/rescue/ensure/when align with their parent
                // construct (if/begin/case), not with a 2-space increment. IndentationWidth
                // does not govern their alignment — ElseAlignment/CaseIndentation do.
                let is_branch_keyword = trimmed_line.starts_with("elsif ")
                    || trimmed_line == "elsif"
                    || trimmed_line == "else"
                    || trimmed_line.starts_with("else ")  // `else # comment`
                    || trimmed_line == "rescue"
                    || trimmed_line.starts_with("rescue ")
                    || trimmed_line == "ensure"
                    || trimmed_line.starts_with("ensure ")
                    || trimmed_line.starts_with("when ");
                let is_closing_token = is_end_keyword
                    || trimmed_line.starts_with(']')
                    || trimmed_line.starts_with(')')
                    || trimmed_line.starts_with('}')
                    || is_branch_keyword;

                // Skip when this line is inside a previously-accepted odd-indent region
                // (e.g. subsequent lines of a do-block body or multiline string whose
                // first line was accepted via is_bracket_continuation / is_dot_chain etc.).
                let in_accepted_region = accepted_odd_indent.map_or(false, |ai| spaces >= ai);

                let flagged = !is_comma_continuation && !is_bracket_continuation
                    && !is_backslash_continuation && !is_boolean_continuation
                    && !is_operator_continuation
                    && !is_dot_chain_continuation && !is_ternary_continuation
                    && !is_begin_continuation
                    && !is_method_chain && !is_after_branch_keyword
                    && !is_closing_token && inline_cond_depth == 0
                    && entering_depth == 0  // skip lines inside bracket expressions
                    && !in_accepted_region;

                if flagged {
                    let start = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Indentation width must be a multiple of 2 (got {effective_spaces})."),
                        range: TextRange::new(start, start + spaces as u32),
                        severity: Severity::Warning,
                    });
                } else if accepted_odd_indent.is_none() {
                    // This line was accepted despite odd indentation — record the level
                    // so subsequent lines in the same block are also accepted.
                    accepted_odd_indent = Some(spaces);
                }
            }

            // Decrement depth when we hit the `end` closing the inline conditional
            let trimmed = line.trim();
            if inline_cond_depth > 0 && (trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end.")) {
                inline_cond_depth -= 1;
            }

            // If this line opened a heredoc, enter heredoc mode for subsequent lines.
            if let Some(term) = new_heredoc_terminator {
                in_heredoc = true;
                heredoc_terminator = term;
            }

            prev_nonempty_line = line;
        }
        diags
    }
}

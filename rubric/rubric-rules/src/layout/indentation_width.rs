use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IndentationWidth;

impl Rule for IndentationWidth {
    fn name(&self) -> &'static str {
        "Layout/IndentationWidth"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut prev_nonempty_line: &str = "";
        // Track when we're inside an inline conditional (x = if / x = unless / x = case)
        // whose continuation lines have alignment-based indentation.
        let mut inline_cond_depth: usize = 0;
        // Track unclosed bracket depth across lines. When > 0 we are inside a bracket
        // expression and continuation lines use alignment indentation — skip odd-indent check.
        let mut bracket_depth: i32 = 0;

        for (i, line) in ctx.lines.iter().enumerate() {
            if line.is_empty() {
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

            // Capture the bracket depth at the START of this line (before updating it from
            // this line's content). Lines inside a bracket expression use alignment
            // indentation and must not be flagged for odd indent counts.
            let entering_depth = bracket_depth;

            // Update bracket depth from this line's content (simplified scan — ignores
            // string literals, but string-literal brackets don't typically cause alignment
            // indentation so the approximation is acceptable).
            for &b in line.as_bytes() {
                match b {
                    b'[' | b'(' => bracket_depth += 1,
                    b']' | b')' => {
                        bracket_depth -= 1;
                        if bracket_depth < 0 {
                            bracket_depth = 0;
                        }
                    }
                    b'#' => break, // stop at start of comment
                    _ => {}
                }
            }

            // Update inline conditional depth tracking
            if line.contains(" = if ") || line.contains(" = unless ") || line.contains(" = case ")
                || line.contains(" << if ") || line.contains(" << unless ") || line.contains(" << case ")
            {
                inline_cond_depth += 1;
            }

            let spaces = line.len() - line.trim_start_matches(' ').len();
            if spaces > 0 && spaces % 2 != 0 {
                // Skip continuation lines — trailing comma means aligned argument continuation.
                // Also skip lines inside inline conditional expressions (alignment to `if` keyword).
                let prev_trim = prev_nonempty_line.trim_end();
                let is_comma_continuation = prev_trim.ends_with(',');
                // Skip when previous line ends with an opening bracket/delimiter —
                // the next line uses alignment indentation to match the bracket position.
                let is_bracket_continuation = prev_trim.ends_with('[')
                    || prev_trim.ends_with('(')
                    || prev_trim.ends_with('{')
                    || prev_trim.ends_with('|');  // block params: `do |x|` / `{|x|`
                // Skip closing tokens — they align with their opener, which may be odd-indented.
                let trimmed_line = line.trim_start();
                let is_end_keyword = trimmed_line == "end"
                    || trimmed_line.starts_with("end ")
                    || trimmed_line.starts_with("end.")
                    || trimmed_line.starts_with("end\n");
                let is_closing_token = is_end_keyword
                    || trimmed_line.starts_with(']')
                    || trimmed_line.starts_with(')')
                    || trimmed_line.starts_with('}');
                if !is_comma_continuation && !is_bracket_continuation
                    && !is_closing_token && inline_cond_depth == 0
                    && entering_depth == 0  // skip lines inside bracket expressions
                {
                    let start = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Indentation width must be a multiple of 2 (got {spaces})."),
                        range: TextRange::new(start, start + spaces as u32),
                        severity: Severity::Warning,
                    });
                }
            }

            // Decrement depth when we hit the `end` closing the inline conditional
            let trimmed = line.trim();
            if inline_cond_depth > 0 && (trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end.")) {
                inline_cond_depth -= 1;
            }

            prev_nonempty_line = line;
        }
        diags
    }
}

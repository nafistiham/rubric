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
            let trimmed = line.trim_end();

            // Check if line ends with `{` (block opener at end of line)
            if trimmed.ends_with('{') {
                // Find the `{` position
                let brace_pos = trimmed.rfind('{').expect("just confirmed it ends with {");
                let before_brace = &trimmed[..brace_pos].trim_end();
                // Only flag if there's a method call before the `{`
                // (i.e., line doesn't just start with `{`)
                if !before_brace.is_empty() {
                    // Skip if `{` is in a hash-literal context.
                    // Hash context: last non-space char before `{` is `=`, `:`, `,`, `(`, `[`, or `{`.
                    let last_char = before_brace.trim_end().chars().last();
                    let is_hash_context = matches!(
                        last_char,
                        Some('=') | Some(':') | Some(',') | Some('(') | Some('[') | Some('{')
                    );
                    if is_hash_context {
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

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AmbiguousBlockAssociation;

impl Rule for AmbiguousBlockAssociation {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousBlockAssociation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect pattern: `word word { ` where we have two bare method calls
            // with a block attached (ambiguous whether block belongs to first or second)
            // Simplified: `\w+ \w+ {` pattern where neither word is a keyword
            // and the first word has no `(` before `\w+`
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;

            while j < len {
                if bytes[j].is_ascii_alphabetic() || bytes[j] == b'_' {
                    // Read first word
                    let w1_start = j;
                    while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                        j += 1;
                    }
                    let w1_end = j;

                    // Check it's preceded by start-of-content or space (not a dot)
                    let before_w1_ok = w1_start == 0 || bytes[w1_start - 1] == b' ';
                    if !before_w1_ok { continue; }

                    // Skip space
                    if j >= len || bytes[j] != b' ' { continue; }
                    j += 1;

                    // Read second word
                    if j >= len || (!bytes[j].is_ascii_alphabetic() && bytes[j] != b'_') { continue; }
                    while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                        j += 1;
                    }

                    // Skip space then check for `{`
                    if j >= len || bytes[j] != b' ' { continue; }
                    j += 1;
                    if j >= len || bytes[j] != b'{' { continue; }

                    // This looks like `method1 method2 {` — ambiguous
                    let first_word = &line[w1_start..w1_end];
                    // Skip Ruby keywords
                    let keywords = ["if", "unless", "while", "until", "return", "do",
                                    "rescue", "ensure", "and", "or", "not", "in", "then",
                                    "begin", "end", "class", "module", "def"];
                    if keywords.contains(&first_word) {
                        continue;
                    }

                    let line_start = ctx.line_start_offsets[i] as usize;
                    let brace_pos = (line_start + j) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Ambiguous block association; use parentheses to disambiguate.".into(),
                        range: TextRange::new(brace_pos, brace_pos + 1),
                        severity: Severity::Warning,
                    });
                    break; // one per line
                }
                j += 1;
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TrailingBodyOnMethodDefinition;

impl TrailingBodyOnMethodDefinition {
    fn indent_of(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }

    /// Returns `true` when a `def` line has a body on the same line as the
    /// `def` keyword, but the closing `end` is on the NEXT line.
    ///
    /// Pattern (multi-line): `def <name>[(<params>)]; <body>`  (no `end` yet)
    ///
    /// Single-line methods (`def foo; body; end` — body AND end on same line)
    /// are NOT flagged here; that is the domain of `Style/SingleLineMethods`.
    /// Empty methods (`def foo; end`) are also skipped.
    fn has_trailing_body(trimmed: &str) -> bool {
        if !trimmed.starts_with("def ") {
            return false;
        }

        // Strip inline comment so `def foo; end  # comment` is recognised
        // as a single-line method the same way `def foo; end` would be.
        let code = Self::strip_inline_comment(trimmed);

        // Must have a semicolon (body separator on the def line).
        if !code.contains(';') {
            return false;
        }

        // Single-line methods (body AND closing `end` on the same line) are
        // handled by Style/SingleLineMethods, not this cop — skip them.
        if code.ends_with(" end") || code.ends_with(";end") {
            return false;
        }

        // Skip endless method definitions: `def foo = expr` or `def foo(p) = expr`
        // These use `=` as the body assignment. The `;` we detected is inside the
        // body expression (e.g. `def m(a, b) = (x = a ; b)`), not a body separator.
        // Heuristic: if the portion before the first `;` contains ` = ` (not `==`,
        // `!=`, `<=`, `>=`, `=~`, `=>`) that follows `)` or an identifier, it's endless.
        {
            let first_semi = code.find(';').unwrap_or(code.len());
            let before_semi = &code[..first_semi];
            if Self::has_endless_marker(before_semi) {
                return false;
            }
        }

        // There must be non-empty, non-comment content after the first `;`.
        // `def foo; # comment` has no body on this line.
        let after_semi = code.splitn(2, ';').nth(1).unwrap_or("").trim();
        !after_semi.is_empty() && !after_semi.starts_with('#')
    }

    /// Returns true if `s` (the portion of a `def` line before the first `;`)
    /// contains the endless-method body marker: a lone `=` (not `==`, `!=`, `<=`,
    /// `>=`, `=~`, `=>`) that appears at parenthesis depth 0 (outside param list).
    fn has_endless_marker(s: &str) -> bool {
        let bytes = s.as_bytes();
        let n = bytes.len();
        let mut depth: i32 = 0;
        let mut i = 0;
        while i < n {
            match bytes[i] {
                b'(' => { depth += 1; }
                b')' => { depth -= 1; }
                b'=' if depth == 0 => {
                    let prev = if i > 0 { bytes[i-1] } else { 0 };
                    let next = if i + 1 < n { bytes[i+1] } else { 0 };
                    // Must be a lone `=`: not `==`, `!=`, `<=`, `>=`, `=~`, `=>`
                    if next != b'=' && next != b'~' && next != b'>'
                        && prev != b'!' && prev != b'<' && prev != b'>' && prev != b'='
                    {
                        // Must follow `)`, space, or identifier — the endless marker
                        if prev == b')' || prev == b' ' || prev == b'\t' {
                            return true;
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// Strip a trailing `# ...` comment from the code portion of a line.
    /// Respects string literals so `#` inside strings is not treated as a comment.
    fn strip_inline_comment(s: &str) -> &str {
        let bytes = s.as_bytes();
        let mut in_single = false;
        let mut in_double = false;
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' if in_single || in_double => { i += 2; continue; }
                b'\'' if !in_double => { in_single = !in_single; }
                b'"'  if !in_single => { in_double = !in_double; }
                b'#'  if !in_single && !in_double => return s[..i].trim_end(),
                _ => {}
            }
            i += 1;
        }
        s.trim_end()
    }
}

impl Rule for TrailingBodyOnMethodDefinition {
    fn name(&self) -> &'static str {
        "Style/TrailingBodyOnMethodDefinition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if !Self::has_trailing_body(trimmed) {
                continue;
            }

            let indent = Self::indent_of(line);
            let line_start = ctx.line_start_offsets[i] as usize;
            let pos = (line_start + indent) as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Do not place the body of a method definition on the same line as the def keyword.".into(),
                range: TextRange::new(pos, pos + trimmed.len() as u32),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

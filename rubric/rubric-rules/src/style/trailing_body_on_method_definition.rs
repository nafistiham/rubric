use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TrailingBodyOnMethodDefinition;

impl TrailingBodyOnMethodDefinition {
    fn indent_of(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }

    /// Returns `true` when the line is a `def` that has a body (code between
    /// the signature and `end`) on the same line, separated by `;`.
    ///
    /// Pattern: `def <name>[(<params>)]; <body>; end`
    ///
    /// An empty method (`def foo; end`) is intentionally excluded because
    /// RuboCop's `TrailingBodyOnMethodDefinition` does not flag it (that is the
    /// domain of `Style/SingleLineMethods` with `AllowIfMethodIsEmpty: true`).
    fn has_trailing_body(trimmed: &str) -> bool {
        if !trimmed.starts_with("def ") {
            return false;
        }

        // Must have a semicolon and `end` on the same line.
        if !trimmed.contains(';') {
            return false;
        }

        // The line must end with `end` (with possible trailing whitespace
        // already removed by `trim_start`, but `trimmed` is the whole line's
        // trim so trailing spaces are gone too).
        if !trimmed.ends_with(" end") && !trimmed.ends_with(";end") {
            return false;
        }

        // Strip the trailing `end` (4 chars: ` end` or `;end`).
        let without_end = &trimmed[..trimmed.len() - 4];
        // Strip the trailing semicolon/space that precedes `end`.
        let before_end = without_end.trim_end_matches(|c: char| c == ';' || c == ' ');

        // If what remains equals the `def ...` signature (no body tokens),
        // then the method is empty: `def foo; end` → skip.
        // A semicolon after the signature is the separator; if stripping it
        // leaves us with a string without any further semicolons that would
        // mean there is body content, check for at least one more `;`.
        before_end.contains(';')
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

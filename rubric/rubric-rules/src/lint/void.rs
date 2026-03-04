use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct Void;

impl Rule for Void {
    fn name(&self) -> &'static str {
        "Lint/Void"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') || t.is_empty() {
                continue;
            }

            // Skip lines that start with keywords or assignments
            if t.starts_with("if ") || t.starts_with("unless ") || t.starts_with("while ")
                || t.starts_with("until ") || t.starts_with("def ") || t.starts_with("class ")
                || t.starts_with("module ") || t.starts_with("return ")
                || t.starts_with("raise ") || t.starts_with("puts ")
                || t.starts_with("print ") || t.starts_with("p ") || t == "end"
                || t == "begin" || t.starts_with("require ")
                || t.starts_with("rescue") || t.starts_with("ensure")
                || t.starts_with("else") || t.starts_with("elsif ")
                // `end % n` — chained method/operator on block result (not standalone void)
                || t.starts_with("end ")
                // String literal — implicit return value or format string (`'...' % args`)
                || t.starts_with('\'') || t.starts_with('"')
            {
                continue;
            }

            // Skip if the next non-empty, non-comment line is `end` — implicit return value
            let next_code = ctx.lines[i+1..].iter()
                .map(|l| l.trim())
                .find(|l| !l.is_empty() && !l.starts_with('#'));
            if next_code.map(|l| l == "end" || l.starts_with("end ") || l.starts_with("end.")).unwrap_or(false) {
                continue;
            }

            // Skip assignments (contains `=` not preceded by comparison operators)
            if t.contains(" = ") || t.contains(" += ") || t.contains(" -= ")
                || t.contains(" *= ") || t.contains(" /= ") || t.contains(" ||= ")
                || t.contains(" &&= ") {
                continue;
            }

            // Skip method calls that look like they have side effects (have parens or receiver)
            if t.contains('(') || t.contains('.') || t.contains('!') {
                continue;
            }

            // Skip lines with a comma — these are method calls with multiple arguments,
            // e.g. `assert_equal a + b, 10`. The comma disambiguates from a standalone
            // void expression like `x + 1`.
            if t.contains(',') {
                continue;
            }

            // Detect standalone arithmetic/comparison: `x + 1`, `y * 3`
            // Pattern: simple expression with arithmetic operator, no assignment
            let has_arithmetic = t.contains(" + ") || t.contains(" - ") || t.contains(" * ")
                || t.contains(" / ") || t.contains(" % ")
                || t.contains(" == ") || t.contains(" != ")
                || t.contains(" > ") || t.contains(" < ")
                || t.contains(" >= ") || t.contains(" <= ");

            if has_arithmetic {
                // Additional guard: if there are more than 2 spaces in the expression,
                // it likely has more than 2 tokens around an operator and is probably
                // a method call with an arithmetic argument rather than a standalone
                // void expression. A simple void like `x + 1` has exactly 2 spaces.
                let space_count = t.chars().filter(|&c| c == ' ').count();
                if space_count > 2 {
                    continue;
                }

                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Expression result is unused (void context).".into(),
                    range: TextRange::new(pos, pos + t.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

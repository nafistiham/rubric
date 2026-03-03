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

            // Detect standalone arithmetic/comparison: `x + 1`, `y * 3`
            // Pattern: simple expression with arithmetic operator, no assignment
            let has_arithmetic = t.contains(" + ") || t.contains(" - ") || t.contains(" * ")
                || t.contains(" / ") || t.contains(" % ")
                || t.contains(" == ") || t.contains(" != ")
                || t.contains(" > ") || t.contains(" < ")
                || t.contains(" >= ") || t.contains(" <= ");

            if has_arithmetic {
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

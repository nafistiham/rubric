use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLinesAroundBlockBody;

impl Rule for EmptyLinesAroundBlockBody {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundBlockBody"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track block openers: true = do-block, false = other (def/class/module/if/etc.)
        let mut block_stack: Vec<bool> = Vec::new();

        for i in 0..n {
            let trimmed = lines[i].trim();

            // Detect `do` at end of line or `do |...|` — block opened with `do`
            let opens_do_block = trimmed.ends_with(" do")
                || trimmed == "do"
                || trimmed.contains(" do |")
                || trimmed.contains(" do|");

            // Detect non-do openers that consume an `end`
            let opens_other = !opens_do_block && (
                trimmed.starts_with("def ")
                    || trimmed == "def"
                    || trimmed.starts_with("class ")
                    || trimmed.starts_with("module ")
                    || trimmed.starts_with("if ")
                    || trimmed.starts_with("unless ")
                    || trimmed.starts_with("while ")
                    || trimmed.starts_with("until ")
                    || trimmed == "begin"
                    || trimmed.starts_with("begin ")
                    || trimmed.starts_with("case ")
            );

            if opens_do_block {
                block_stack.push(true);
                // Check if the next line is blank
                if i + 1 < n && lines[i + 1].trim().is_empty() {
                    let blank_line = i + 1;
                    let line_start = ctx.line_start_offsets[blank_line];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra empty line detected after `do` block opening.".into(),
                        range: TextRange::new(line_start, line_start + lines[blank_line].len() as u32),
                        severity: Severity::Warning,
                    });
                }
            } else if opens_other {
                block_stack.push(false);
            }

            // Detect `end` line — only flag blank-before-end for do-blocks
            if trimmed == "end" {
                let is_do_block = block_stack.pop().unwrap_or(false);
                if is_do_block && i > 0 && lines[i - 1].trim().is_empty() {
                    let blank_line = i - 1;
                    let line_start = ctx.line_start_offsets[blank_line];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra empty line detected before block `end`.".into(),
                        range: TextRange::new(line_start, line_start + lines[blank_line].len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

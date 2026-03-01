use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UselessElseWithoutRescue;

impl Rule for UselessElseWithoutRescue {
    fn name(&self) -> &'static str {
        "Lint/UselessElseWithoutRescue"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let t = lines[i].trim();
            if t == "begin" {
                // Scan the begin block for rescue and else
                let _begin_line = i;
                let mut has_rescue = false;
                let mut else_line: Option<usize> = None;
                let mut depth = 1usize;
                let mut j = i + 1;

                while j < n && depth > 0 {
                    let tl = lines[j].trim();
                    if tl == "begin" || tl.starts_with("if ") || tl.starts_with("unless ")
                        || tl.starts_with("while ") || tl.starts_with("until ")
                        || tl.starts_with("def ") || tl.starts_with("class ")
                        || tl.starts_with("module ") {
                        depth += 1;
                    }
                    if tl == "end" { depth -= 1; }
                    if depth == 1 {
                        if tl.starts_with("rescue") { has_rescue = true; }
                        if tl == "else" && else_line.is_none() { else_line = Some(j); }
                    }
                    j += 1;
                }

                if let Some(el) = else_line {
                    if !has_rescue {
                        let indent = lines[el].len() - lines[el].trim_start().len();
                        let line_start = ctx.line_start_offsets[el] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "`else` in `begin` block has no effect without `rescue`.".into(),
                            range: TextRange::new(pos, pos + 4),
                            severity: Severity::Warning,
                        });
                    }
                }

                i = j;
                continue;
            }
            i += 1;
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HashAlignment;

impl Rule for HashAlignment {
    fn name(&self) -> &'static str {
        "Layout/HashAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut i = 0;
        while i < n {
            // Find the start of a group: consecutive lines containing `=>`
            if !lines[i].contains("=>") {
                i += 1;
                continue;
            }

            // Collect the group
            let group_start = i;
            while i < n && lines[i].contains("=>") {
                i += 1;
            }
            let group_end = i; // exclusive

            if group_end - group_start < 2 {
                // Single line group — nothing to align
                continue;
            }

            // Find the column of `=>` in each line of the group
            let columns: Vec<Option<usize>> = (group_start..group_end)
                .map(|li| lines[li].find("=>"))
                .collect();

            // The reference column is the first line's `=>` position
            let reference = match columns[0] {
                Some(c) => c,
                None => continue,
            };

            // Flag lines where `=>` appears at a different column
            for (k, col) in columns.iter().enumerate() {
                if let Some(c) = col {
                    if *c != reference {
                        let line_idx = group_start + k;
                        let line_start = ctx.line_start_offsets[line_idx] as usize;
                        let pos = (line_start + c) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "Hash rocket `=>` is not aligned (column {}, expected {}).",
                                c, reference
                            ),
                            range: TextRange::new(pos, pos + 2),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }
}

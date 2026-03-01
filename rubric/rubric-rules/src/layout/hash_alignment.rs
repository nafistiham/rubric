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

        // Returns the byte position of `=>` in a line, skipping comments and strings.
        // Returns None if the line is a comment or has no `=>` outside strings/comments.
        let rocket_col = |line: &str| -> Option<usize> {
            let trimmed = line.trim_start();
            // Skip pure comment lines
            if trimmed.starts_with('#') {
                return None;
            }
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;
            while j < len {
                let b = bytes[j];
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment — stop
                    None => {}
                }
                if j + 1 < len && bytes[j] == b'=' && bytes[j + 1] == b'>' {
                    return Some(j);
                }
                j += 1;
            }
            None
        };

        let mut i = 0;
        while i < n {
            // Find the start of a group: consecutive lines containing `=>` (outside strings/comments)
            if rocket_col(&lines[i]).is_none() {
                i += 1;
                continue;
            }

            // Collect the group
            let group_start = i;
            while i < n && rocket_col(&lines[i]).is_some() {
                i += 1;
            }
            let group_end = i; // exclusive

            if group_end - group_start < 2 {
                // Single line group — nothing to align
                continue;
            }

            // Find the column of `=>` in each line of the group
            let columns: Vec<Option<usize>> = (group_start..group_end)
                .map(|li| rocket_col(&lines[li]))
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

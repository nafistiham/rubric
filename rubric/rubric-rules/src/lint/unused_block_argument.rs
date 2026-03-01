use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UnusedBlockArgument;

impl Rule for UnusedBlockArgument {
    fn name(&self) -> &'static str {
        "Lint/UnusedBlockArgument"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();

            // Detect `do |param|` pattern
            let do_pos = trimmed.find(" do |").or_else(|| {
                if trimmed.starts_with("do |") { Some(0) } else { None }
            });

            if let Some(do_offset) = do_pos {
                let pipe_start = if do_offset == 0 {
                    trimmed.find('|').map(|p| p)
                } else {
                    trimmed.find(" do |").map(|p| p + " do ".len())
                };

                if let Some(pipe_open) = pipe_start {
                    if let Some(pipe_close) = trimmed[pipe_open + 1..].find('|') {
                        let params_str = &trimmed[pipe_open + 1..pipe_open + 1 + pipe_close];
                        let params: Vec<&str> = params_str
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect();

                        // Find the end of this block
                        let mut depth = 1usize;
                        let mut j = i + 1;
                        let block_start = i;
                        while j < n && depth > 0 {
                            let t = lines[j].trim();
                            if t.contains(" do") || t == "do" { depth += 1; }
                            if t == "end" { depth -= 1; }
                            j += 1;
                        }
                        let block_end = j;

                        // Collect block body source
                        let body: String = lines[i + 1..block_end.min(n)]
                            .iter()
                            .map(|l| l.as_ref())
                            .collect::<Vec<_>>()
                            .join("\n");

                        for param in params {
                            // Skip already-prefixed-with-underscore params
                            if param.starts_with('_') || param.is_empty() {
                                continue;
                            }
                            // Check if param is used in body
                            if !body_uses_var(&body, param) {
                                let indent = line.len() - trimmed.len();
                                let line_start = ctx.line_start_offsets[block_start] as usize;
                                let pos = (line_start + indent) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!(
                                        "Block argument `{}` is unused; prefix with `_` to suppress.",
                                        param
                                    ),
                                    range: TextRange::new(pos, pos + 2),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
            }

            i += 1;
        }

        diags
    }
}

fn body_uses_var(body: &str, var: &str) -> bool {
    let vb = var.as_bytes();
    let bb = body.as_bytes();
    let n = bb.len();
    let vn = vb.len();

    let mut pos = 0;
    while pos + vn <= n {
        if &bb[pos..pos + vn] == vb {
            let before_ok = pos == 0 || !bb[pos - 1].is_ascii_alphanumeric() && bb[pos - 1] != b'_';
            let after_ok = pos + vn >= n || !bb[pos + vn].is_ascii_alphanumeric() && bb[pos + vn] != b'_';
            if before_ok && after_ok {
                return true;
            }
        }
        pos += 1;
    }
    false
}

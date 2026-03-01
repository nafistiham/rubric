use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantSelf;

impl Rule for RedundantSelf {
    fn name(&self) -> &'static str {
        "Style/RedundantSelf"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Only flag `self.method` inside def..end blocks
        // Track if we're inside a def
        let mut in_def = false;
        let mut depth = 0usize;

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            if trimmed.starts_with("def ") || trimmed == "def" {
                in_def = true;
                depth += 1;
            } else if trimmed.starts_with("class ") || trimmed.starts_with("module ")
                || trimmed.starts_with("if ") || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ") || trimmed.starts_with("until ")
                || trimmed.starts_with("for ") || trimmed.starts_with("begin")
                || trimmed == "do" || trimmed.ends_with(" do") {
                depth += 1;
            } else if trimmed == "end" || trimmed.starts_with("end ") {
                if depth > 0 { depth -= 1; }
                if depth == 0 { in_def = false; }
            }

            if in_def {
                // Scan for `self.method_name` (not followed by `=`)
                let bytes = line.as_bytes();
                let len = bytes.len();
                let pattern = b"self.";
                let pat_len = pattern.len();
                let mut j = 0;
                while j + pat_len <= len {
                    if &bytes[j..j + pat_len] == pattern {
                        // Check word boundary before `self`
                        let before_ok = j == 0 || (!bytes[j - 1].is_ascii_alphanumeric() && bytes[j - 1] != b'_');

                        if before_ok {
                            // Check what follows the method name (skip past `self.method_name`)
                            let mut k = j + pat_len;
                            while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_' || bytes[k] == b'?' || bytes[k] == b'!') {
                                k += 1;
                            }
                            // Skip if it's assignment (`self.foo =`)
                            let is_assignment = k < len && bytes[k] == b' '
                                && k + 1 < len && bytes[k + 1] == b'=';
                            // Skip `self.class` which is special
                            let method_name = &line[j + pat_len..k];
                            let is_special = method_name == "class" || method_name == "is_a?" || method_name == "send";

                            if !is_assignment && !is_special {
                                let line_start = ctx.line_start_offsets[i];
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Redundant `self.` in method call.".into(),
                                    range: TextRange::new(line_start + j as u32, line_start + (j + pat_len) as u32),
                                    severity: Severity::Warning,
                                });
                            }
                            j = k;
                            continue;
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }

        diags
    }
}

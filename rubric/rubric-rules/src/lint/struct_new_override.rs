use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct StructNewOverride;

impl Rule for StructNewOverride {
    fn name(&self) -> &'static str {
        "Lint/StructNewOverride"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let trimmed = lines[i].trim();
            // Look for `Struct.new(:member)` lines
            if trimmed.contains("Struct.new(") {
                // Extract member names from Struct.new(:foo, :bar)
                let struct_pos = trimmed.find("Struct.new(").unwrap_or(0);
                let after = &trimmed[struct_pos + "Struct.new(".len()..];
                let paren_close = after.find(')').unwrap_or(after.len());
                let members_str = &after[..paren_close];
                let members: Vec<&str> = members_str
                    .split(',')
                    .map(|s| s.trim().trim_start_matches(':'))
                    .collect();

                // Look ahead for `do ... def <member> ... end ... end`
                let mut j = i + 1;
                let mut depth = 0usize;
                while j < n {
                    let t = lines[j].trim();
                    if t == "do" || t.ends_with(" do") { depth += 1; }
                    if t == "end" {
                        if depth == 0 { break; }
                        depth -= 1;
                    }
                    if t.starts_with("def ") {
                        let method_name_part = &t["def ".len()..];
                        let method_end = method_name_part
                            .find(|c: char| c == '(' || c == ' ')
                            .unwrap_or(method_name_part.len());
                        let method_name = &method_name_part[..method_end];
                        if members.contains(&method_name) {
                            let indent = lines[j].len() - lines[j].trim_start().len();
                            let line_start = ctx.line_start_offsets[j] as usize;
                            let pos = (line_start + indent) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Method `{}` overrides a Struct member accessor.",
                                    method_name
                                ),
                                range: TextRange::new(pos, pos + t.len() as u32),
                                severity: Severity::Warning,
                            });
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

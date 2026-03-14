use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UselessAccessModifier;

/// Returns true if the trimmed line is a standalone access modifier statement
/// (not `private def foo` or `private :foo`).
fn standalone_modifier(trimmed: &str) -> Option<&'static str> {
    match trimmed {
        "private" => Some("private"),
        "public" => Some("public"),
        "protected" => Some("protected"),
        _ => None,
    }
}

/// Returns true if the trimmed line starts a method definition.
fn is_def_line(trimmed: &str) -> bool {
    trimmed.starts_with("def ") || trimmed == "def"
}

/// Returns true if the trimmed line is `end` (closing a class/module scope).
fn is_scope_end(trimmed: &str) -> bool {
    trimmed == "end"
}

impl Rule for UselessAccessModifier {
    fn name(&self) -> &'static str {
        "Lint/UselessAccessModifier"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Track the last standalone modifier seen: (line_index, modifier_name, line_start_offset, indent)
        let mut pending: Option<(usize, &'static str, usize, usize)> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip blank lines and comments — they don't affect modifier state
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let indent = line.len() - trimmed.len();

            if let Some(modifier) = standalone_modifier(trimmed) {
                // We hit a new standalone modifier.
                // If there's already a pending modifier with no def between them, flag the pending one.
                if let Some((prev_i, prev_mod, prev_offset, prev_indent)) = pending {
                    let _ = prev_i;
                    let start = (prev_offset + prev_indent) as u32;
                    let end = start + prev_mod.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Useless `{}` access modifier.",
                            prev_mod
                        ),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                }
                pending = Some((i, modifier, line_start, indent));
            } else if is_def_line(trimmed) {
                // A real method definition follows the modifier — it was useful.
                pending = None;
            } else if is_scope_end(trimmed) {
                // We hit an `end` — if there's a pending modifier, flag it as useless
                // (no method definition followed it before the scope closed).
                if let Some((_, prev_mod, prev_offset, prev_indent)) = pending {
                    let start = (prev_offset + prev_indent) as u32;
                    let end = start + prev_mod.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Useless `{}` access modifier.",
                            prev_mod
                        ),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                    pending = None;
                }
            }
            // All other lines (method calls, assignments, etc.) do not clear the pending
            // modifier — only an actual `def` clears it.
        }

        // End of file: if there's still a pending modifier, flag it.
        if let Some((_, prev_mod, prev_offset, prev_indent)) = pending {
            let start = (prev_offset + prev_indent) as u32;
            let end = start + prev_mod.len() as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: format!(
                    "Useless `{}` access modifier.",
                    prev_mod
                ),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

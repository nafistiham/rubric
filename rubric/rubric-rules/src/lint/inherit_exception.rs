use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct InheritException;

/// Returns the index of the comment character `#` on the line, ignoring
/// `#` that appear inside string literals.
/// Returns `None` if no real comment exists.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        i += 1;
    }
    None
}

impl Rule for InheritException {
    fn name(&self) -> &'static str {
        "Lint/InheritException"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Only scan up to the real comment character
            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];

            // Match `class Foo < Exception` or `class Foo < ::Exception`
            // We look for `< Exception` or `< ::Exception` preceded by class syntax.
            if let Some(class_pos) = scan_slice.find("class ") {
                // Find `<` after the class name
                let after_class = &scan_slice[class_pos + 6..];
                if let Some(lt_pos) = after_class.find('<') {
                    let after_lt = after_class[lt_pos + 1..].trim_start();
                    // Strip optional `::` namespace prefix
                    let parent = after_lt.strip_prefix("::").unwrap_or(after_lt);
                    // Check if parent starts with `Exception` followed by word boundary
                    if parent.starts_with("Exception") {
                        let next = parent.as_bytes().get(9).copied();
                        let at_boundary = next.map_or(true, |b| {
                            !b.is_ascii_alphanumeric() && b != b'_'
                        });
                        if at_boundary {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            let start = line_start as u32;
                            let end = (line_start + line.len()) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Avoid inheriting from the Exception class. Inherit from StandardError or its subclasses.".into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        diags
    }
}

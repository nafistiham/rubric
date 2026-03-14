use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct OpenStructUse;

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

impl Rule for OpenStructUse {
    fn name(&self) -> &'static str {
        "Style/OpenStructUse"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Patterns to detect: `OpenStruct.new` and `require 'ostruct'` / `require "ostruct"`
        let patterns: &[&str] = &[
            "OpenStruct.new",
            "require 'ostruct'",
            "require \"ostruct\"",
        ];

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Only scan up to the real comment character
            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];

            let line_start = ctx.line_start_offsets[i] as usize;

            for pattern in patterns {
                let mut search = 0usize;
                while search < scan_slice.len() {
                    if let Some(rel) = scan_slice[search..].find(pattern) {
                        let abs = search + rel;
                        let start = (line_start + abs) as u32;
                        let end = (line_start + abs + pattern.len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Avoid using OpenStruct; use Struct or a simple class instead.".into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                        search = abs + pattern.len();
                    } else {
                        break;
                    }
                }
            }
        }

        diags
    }
}

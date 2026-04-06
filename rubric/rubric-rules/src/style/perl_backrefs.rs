use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct PerlBackrefs;

/// Extract heredoc terminator from a line (`<<WORD`, `<<-WORD`, `<<~WORD`).
fn heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let rest = &line[i + 2..];
            let rest = rest.strip_prefix('-').or_else(|| rest.strip_prefix('~')).unwrap_or(rest);
            let rest = if rest.starts_with('"') || rest.starts_with('\'') || rest.starts_with('`') {
                &rest[1..]
            } else {
                rest
            };
            let word: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !word.is_empty() {
                return Some(word);
            }
        }
        i += 1;
    }
    None
}

impl Rule for PerlBackrefs {
    fn name(&self) -> &'static str {
        "Style/PerlBackrefs"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // Track heredoc bodies — `$N` inside SQL/shell heredocs is not a Perl backref.
        let mut in_heredoc: Option<String> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines (including the terminator line itself).
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // Detect heredoc opener — body starts on the next line.
            if let Some(term) = heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: still check the opener line itself for `$N` in Ruby code.
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                if let Some(delim) = in_string {
                    match b {
                        b'\\' => {
                            j += 2;
                            continue;
                        }
                        c if c == delim => {
                            in_string = None;
                        }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                match b {
                    b'"' | b'\'' | b'`' => {
                        in_string = Some(b);
                    }
                    b'#' => break, // inline comment — stop
                    b'$' => {
                        // Check if next char is a digit 1-9
                        if j + 1 < n && bytes[j + 1].is_ascii_digit() && bytes[j + 1] != b'0' {
                            let start = (line_start + j) as u32;
                            let end = (line_start + j + 2) as u32;
                            let var_name = format!("${}", bytes[j + 1] as char);
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Prefer `Regexp.last_match({})` over `{}`.",
                                    bytes[j + 1] as char,
                                    var_name
                                ),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                            j += 2;
                            continue;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
        }

        diags
    }
}

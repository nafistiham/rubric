use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantReturn;

/// Extract heredoc terminator from a line (supports <<~TERM, <<-TERM, <<TERM).
fn extract_heredoc_term_rr(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            if j < bytes.len() && (bytes[j] == b'-' || bytes[j] == b'~') { j += 1; }
            if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`') { j += 1; }
            let start = j;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }
        i += 1;
    }
    None
}

/// Returns true if `line` is a `return` statement (not a method call like `return_value`).
fn is_return_statement(line: &str) -> bool {
    if line == "return" { return true; }
    if line.starts_with("return ") || line.starts_with("return\t") { return true; }
    false
}

impl Rule for RedundantReturn {
    fn name(&self) -> &'static str {
        "Style/RedundantReturn"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let mut in_heredoc: Option<String> = None;
        let mut i = 0;
        while i < lines.len() {
            // Heredoc body tracking
            if let Some(ref term) = in_heredoc.clone() {
                if lines[i].trim() == term.as_str() {
                    in_heredoc = None;
                }
                i += 1;
                continue;
            }
            if let Some(term) = extract_heredoc_term_rr(lines[i]) {
                in_heredoc = Some(term);
            }

            let trimmed = lines[i].trim();
            // Find the start of a method definition
            if trimmed.starts_with("def ") || trimmed == "def" {
                let def_line = i;
                let mut depth = 1usize;
                i += 1;
                // Scan to find the matching `end`
                while i < lines.len() && depth > 0 {
                    // Heredoc tracking inside method scan
                    if let Some(ref term) = in_heredoc.clone() {
                        if lines[i].trim() == term.as_str() {
                            in_heredoc = None;
                        }
                        i += 1;
                        continue;
                    }
                    if let Some(term) = extract_heredoc_term_rr(lines[i]) {
                        in_heredoc = Some(term);
                    }
                    let t = lines[i].trim();
                    if t.starts_with("def ") || t.starts_with("class ")
                       || t.starts_with("module ") || t.starts_with("if ")
                       || t.starts_with("unless ") || t.starts_with("while ")
                       || t.starts_with("until ") || t.starts_with("for ")
                       || t == "case" || t.starts_with("case ")
                       || t == "do" || t.ends_with(" do") || t.contains(" do |") || t.contains(" do|")
                       || t.starts_with("begin") {
                        depth += 1;
                    } else if t == "end" {
                        depth -= 1;
                        if depth == 0 {
                            // This `end` closes the `def` — find last content line before it
                            let mut j = i.saturating_sub(1);
                            while j > def_line {
                                let last = lines[j].trim();
                                if !last.is_empty() && !last.starts_with('#') {
                                    if is_return_statement(last) {
                                        // Skip `return x, y` (multiple return values) —
                                        // equivalent to AllowMultipleReturnValues: true.
                                        let after_kw = if last.len() > "return".len() { last["return".len()..].trim_start() } else { "" };
                                        // A comma in the value list signals multiple return values.
                                        // Heuristic: if the part after `return` contains a comma
                                        // and doesn't start with a string/array (where commas are
                                        // inside a single value), treat it as multi-value.
                                        let has_multiple = !after_kw.is_empty()
                                            && after_kw.contains(',')
                                            && !after_kw.starts_with('[')
                                            && !after_kw.starts_with('"')
                                            && !after_kw.starts_with('\'');
                                        if !has_multiple {
                                            let line_start = ctx.line_start_offsets[j];
                                            let return_end = line_start + "return".len() as u32;
                                            diags.push(Diagnostic {
                                                rule: self.name(),
                                                message: "Redundant `return` in last expression of method.".into(),
                                                range: TextRange::new(line_start, return_end),
                                                severity: Severity::Warning,
                                            });
                                        }
                                    }
                                    break;
                                }
                                j -= 1;
                            }
                        }
                    }
                    i += 1;
                }
                continue;
            }
            i += 1;
        }
        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashSet;

pub struct DuplicateRequire;

impl Rule for DuplicateRequire {
    fn name(&self) -> &'static str {
        "Lint/DuplicateRequire"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // Key includes the require kind so `require 'foo'` and
        // `require_relative 'foo'` are treated as distinct — they load
        // different files and flagging them as duplicates is a false positive.
        let mut seen: HashSet<String> = HashSet::new();
        let mut heredoc_term: Option<String> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines — Ruby template generators commonly embed
            // `require` statements in <<~RUBY heredocs, which must not be
            // counted against real requires elsewhere in the same file.
            if let Some(ref term) = heredoc_term {
                if line.trim() == term.as_str() {
                    heredoc_term = None;
                }
                continue;
            }

            // Detect heredoc opener on this line before processing it.
            if let Some(term) = extract_heredoc_terminator(line) {
                heredoc_term = Some(term);
                // Fall through: the opening line itself is real Ruby code.
            }

            let trimmed = line.trim_start();

            // Skip comment lines.
            if trimmed.starts_with('#') {
                continue;
            }

            if let Some(key) = extract_require_key(trimmed) {
                if !seen.insert(key.clone()) {
                    let indent = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Duplicate `require` for `{}`.",
                            key.splitn(2, ':').nth(1).unwrap_or(&key)
                        ),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

/// Returns a unique key for the require call that includes the kind
/// (`require:path` or `require_relative:path`) so the two are never
/// conflated as duplicates of each other.
fn extract_require_key(line: &str) -> Option<String> {
    let prefixes = ["require_relative ", "require "];
    for prefix in &prefixes {
        if line.starts_with(prefix) {
            let rest = line[prefix.len()..].trim();
            if (rest.starts_with('\'') && rest.ends_with('\''))
                || (rest.starts_with('"') && rest.ends_with('"'))
            {
                let path = &rest[1..rest.len() - 1];
                let kind = prefix.trim_end();
                return Some(format!("{}:{}", kind, path));
            }
        }
    }
    None
}

/// If `line` contains a heredoc opener (`<<WORD`, `<<-WORD`, `<<~WORD`),
/// returns the terminator string (e.g. `"WORD"`). Otherwise returns `None`.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let pos = line.find("<<")?;
    let rest = &line[pos + 2..];
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    let rest = rest.strip_prefix('~').unwrap_or(rest);
    // Strip optional surrounding quotes.
    let rest = if rest.starts_with('"') || rest.starts_with('\'') || rest.starts_with('`') {
        &rest[1..]
    } else {
        rest
    };
    let word: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if word.is_empty() { None } else { Some(word) }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashSet;

pub struct DuplicateMagicComment;

/// Recognized magic-comment key prefixes (lowercased, without `# `).
const MAGIC_PREFIXES: &[(&str, &str)] = &[
    ("frozen_string_literal:", "frozen_string_literal"),
    ("encoding:", "encoding"),
    ("coding:", "coding"),
    ("warn_indent:", "warn_indent"),
];

/// Returns the magic-comment key if `line` is a magic comment, otherwise `None`.
fn magic_comment_key(line: &str) -> Option<&'static str> {
    let trimmed = line.trim();
    // Must start with `#`
    if !trimmed.starts_with('#') {
        return None;
    }
    // Strip the leading `#` and optional whitespace
    let after_hash = trimmed[1..].trim_start();
    let lower = after_hash.to_ascii_lowercase();

    for (prefix, key) in MAGIC_PREFIXES {
        if lower.starts_with(prefix) {
            return Some(key);
        }
    }
    None
}

impl Rule for DuplicateMagicComment {
    fn name(&self) -> &'static str {
        "Lint/DuplicateMagicComment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut seen: HashSet<&'static str> = HashSet::new();

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            if let Some(key) = magic_comment_key(line) {
                if !seen.insert(key) {
                    // Already seen this key — flag this duplicate line
                    let line_start = ctx.line_start_offsets[line_idx] as u32;
                    let line_end = line_start + line.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Duplicate magic comment detected.".into(),
                        range: TextRange::new(line_start, line_end),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

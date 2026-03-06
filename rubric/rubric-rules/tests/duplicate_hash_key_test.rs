use rubric_core::walker::walk;
use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_hash_key::DuplicateHashKey;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/duplicate_hash_key/offending.rb");
const CORRECTED: &str = include_str!("fixtures/lint/duplicate_hash_key/corrected.rb");

/// Helper: run the rule through the AST walker (the correct pipeline for
/// `check_node` rules).
fn run(src: &str) -> Vec<rubric_core::Diagnostic> {
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![Box::new(DuplicateHashKey)];
    walk(src.as_bytes(), &ctx, &rules)
}

#[test]
fn detects_duplicate_hash_key() {
    let diags = run(OFFENDING);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateHashKey"));
}

#[test]
fn no_violation_for_unique_hash_keys() {
    let diags = run(CORRECTED);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── Bug 1: string key duplicates must be detected ────────────────────────────
// `{ 'Mention' => :a, 'Mention' => :b }` — two identical string keys.
// The old text-scanner only handled `word:` symbol syntax and missed `"str" =>`
// and `'str' =>` patterns entirely.
#[test]
fn detects_duplicate_string_key() {
    let src = "h = { 'Mention' => :a, 'Mention' => :b }\n";
    let diags = run(src);
    assert!(!diags.is_empty(), "string key duplicate should be detected");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateHashKey"));
}

// ── Bug 2: nested sub-hash keys must not fire ─────────────────────────────────
// `mention: [mention: :status]` — the outer key is `mention`; the inner
// `mention:` is inside an array value `[...]` and is a key of a different
// (implicit) hash at a deeper scope. Must NOT fire.
#[test]
fn no_false_positive_for_nested_same_name_key() {
    let src = "HASH = { mention: [mention: :status], reblog: [reblog: :status] }\n";
    let diags = run(src);
    assert!(diags.is_empty(), "nested same-name keys falsely flagged: {:?}", diags);
}

// ── True positive: symbol key duplicate still detected ───────────────────────
#[test]
fn detects_duplicate_symbol_key() {
    let src = "h = { a: 1, b: 2, a: 3 }\n";
    let diags = run(src);
    assert!(!diags.is_empty(), "duplicate symbol key should be detected");
}

// ── Bug 3: CSS properties inside heredoc values must not fire ─────────────────
// Heredoc bodies like `<<~CSS` contain CSS such as `padding: 0;` which look
// like Ruby symbol keys to the raw-byte scanner. When two heredoc values share
// the same CSS property name, it was falsely flagged as a duplicate hash key.
#[test]
fn no_false_positive_for_css_in_heredoc_values() {
    let src = r#"INLINE_STYLES = {
  blockquote: <<~CSS,
    background: #FCF8FF;
    padding: 0;
  CSS
  status_link: <<~CSS,
    padding: 24px;
    color: #1C1A25;
  CSS
  div_account: <<~CSS,
    color: #787588;
  CSS
}
"#;
    let diags = run(src);
    assert!(
        diags.is_empty(),
        "CSS properties inside heredoc values falsely flagged as duplicate hash keys: {:?}",
        diags
    );
}

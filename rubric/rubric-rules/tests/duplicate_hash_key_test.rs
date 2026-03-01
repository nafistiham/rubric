use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_hash_key::DuplicateHashKey;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/duplicate_hash_key/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/duplicate_hash_key/corrected.rb");

#[test]
fn detects_duplicate_hash_key() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DuplicateHashKey.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateHashKey"));
}

#[test]
fn no_violation_for_unique_hash_keys() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = DuplicateHashKey.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

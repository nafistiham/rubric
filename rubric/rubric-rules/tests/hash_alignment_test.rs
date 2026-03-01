use rubric_core::{LintContext, Rule};
use rubric_rules::layout::hash_alignment::HashAlignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/hash_alignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/hash_alignment/corrected.rb");

#[test]
fn detects_misaligned_hash_rockets() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = HashAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for misaligned rockets, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/HashAlignment"));
}

#[test]
fn no_violation_with_aligned_hash_rockets() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = HashAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

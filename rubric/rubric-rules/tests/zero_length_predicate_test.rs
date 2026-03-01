use rubric_core::{LintContext, Rule};
use rubric_rules::style::zero_length_predicate::ZeroLengthPredicate;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/zero_length_predicate/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/zero_length_predicate/corrected.rb");

#[test]
fn detects_zero_length_check() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ZeroLengthPredicate.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/ZeroLengthPredicate"));
}

#[test]
fn no_violation_for_empty_predicate() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = ZeroLengthPredicate.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

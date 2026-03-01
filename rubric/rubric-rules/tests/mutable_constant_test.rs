use rubric_core::{LintContext, Rule};
use rubric_rules::style::mutable_constant::MutableConstant;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/mutable_constant/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/mutable_constant/corrected.rb");

#[test]
fn detects_mutable_constant() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MutableConstant.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/MutableConstant"));
}

#[test]
fn no_violation_for_frozen_constant() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MutableConstant.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

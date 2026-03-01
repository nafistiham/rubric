use rubric_core::{LintContext, Rule};
use rubric_rules::style::yoda_condition::YodaCondition;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/yoda_condition/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/yoda_condition/corrected.rb");

#[test]
fn detects_yoda_condition() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = YodaCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/YodaCondition"));
}

#[test]
fn no_violation_for_normal_condition() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = YodaCondition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

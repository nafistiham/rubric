use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_return::RedundantReturn;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/redundant_return/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/redundant_return/corrected.rb");

#[test]
fn detects_redundant_return() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantReturn.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for redundant return, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantReturn"));
}

#[test]
fn no_violation_without_redundant_return() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RedundantReturn.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

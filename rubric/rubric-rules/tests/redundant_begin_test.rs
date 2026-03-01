use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_begin::RedundantBegin;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/redundant_begin/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/redundant_begin/corrected.rb");

#[test]
fn detects_redundant_begin_in_method() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantBegin.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantBegin"));
}

#[test]
fn no_violation_without_redundant_begin() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RedundantBegin.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

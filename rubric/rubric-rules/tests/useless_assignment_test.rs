use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_assignment::UselessAssignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/useless_assignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/useless_assignment/corrected.rb");

#[test]
fn detects_useless_assignment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for unused variable, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessAssignment"));
}

#[test]
fn no_violation_with_all_vars_used() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

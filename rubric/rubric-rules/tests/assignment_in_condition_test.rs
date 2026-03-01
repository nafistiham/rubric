use rubric_core::{LintContext, Rule};
use rubric_rules::lint::assignment_in_condition::AssignmentInCondition;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/assignment_in_condition/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/assignment_in_condition/corrected.rb");

#[test]
fn detects_assignment_in_condition() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AssignmentInCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/AssignmentInCondition"));
}

#[test]
fn no_violation_for_comparison() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = AssignmentInCondition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

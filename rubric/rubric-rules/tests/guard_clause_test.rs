use rubric_core::{LintContext, Rule};
use rubric_rules::style::guard_clause::GuardClause;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/guard_clause/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/guard_clause/corrected.rb");

#[test]
fn detects_guard_clause_opportunity() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = GuardClause.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for guard clause pattern, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/GuardClause"));
}

#[test]
fn no_violation_with_guard_clause() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = GuardClause.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

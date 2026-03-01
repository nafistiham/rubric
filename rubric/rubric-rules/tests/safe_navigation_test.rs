use rubric_core::{LintContext, Rule};
use rubric_rules::style::safe_navigation::SafeNavigation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/safe_navigation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/safe_navigation/corrected.rb");

#[test]
fn detects_safe_navigation_opportunity() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for `x && x.foo` pattern, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/SafeNavigation"));
}

#[test]
fn no_violation_with_safe_navigation_operator() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

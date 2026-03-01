use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_comparison::UselessComparison;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/useless_comparison/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/useless_comparison/corrected.rb");

#[test]
fn detects_useless_comparison() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessComparison"));
}

#[test]
fn no_violation_for_different_operands() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UselessComparison.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

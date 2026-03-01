use rubric_core::{LintContext, Rule};
use rubric_rules::lint::unreachable_code::UnreachableCode;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/unreachable_code/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/unreachable_code/corrected.rb");

#[test]
fn detects_unreachable_code() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnreachableCode.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnreachableCode"));
}

#[test]
fn no_violation_for_reachable_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UnreachableCode.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

use rubric_core::{LintContext, Rule};
use rubric_rules::style::raise_args::RaiseArgs;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/raise_args/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/raise_args/corrected.rb");

#[test]
fn detects_raise_with_comma_style() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/RaiseArgs"));
}

#[test]
fn no_violation_for_raise_new_style() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

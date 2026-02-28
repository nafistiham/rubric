use rubric_core::{LintContext, Rule};
use rubric_rules::style::trailing_comma_in_arguments::TrailingCommaInArguments;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/trailing_comma_in_arguments/offending.rb");
const CORRECTED: &str = include_str!("fixtures/style/trailing_comma_in_arguments/corrected.rb");

#[test]
fn detects_trailing_comma_in_args() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TrailingCommaInArguments.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Style/TrailingCommaInArguments"));
}

#[test]
fn no_violation_without_trailing_comma() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = TrailingCommaInArguments.check_source(&ctx);
    assert!(diags.is_empty());
}

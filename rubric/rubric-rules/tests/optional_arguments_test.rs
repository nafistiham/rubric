use rubric_core::{LintContext, Rule};
use rubric_rules::style::optional_arguments::OptionalArguments;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/optional_arguments/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/optional_arguments/corrected.rb");

#[test]
fn detects_required_after_optional_argument() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = OptionalArguments.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/OptionalArguments"));
}

#[test]
fn no_violation_for_correct_argument_order() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = OptionalArguments.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

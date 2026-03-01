use rubric_core::{LintContext, Rule};
use rubric_rules::style::trailing_comma_in_array_literal::TrailingCommaInArrayLiteral;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/trailing_comma_in_array_literal/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/trailing_comma_in_array_literal/corrected.rb");

#[test]
fn detects_trailing_comma_in_array() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TrailingCommaInArrayLiteral.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/TrailingCommaInArrayLiteral"));
}

#[test]
fn no_violation_for_no_trailing_comma_in_array() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = TrailingCommaInArrayLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

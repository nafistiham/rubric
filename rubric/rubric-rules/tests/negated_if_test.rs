use rubric_core::{LintContext, Rule};
use rubric_rules::style::negated_if::NegatedIf;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/negated_if/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/negated_if/corrected.rb");

#[test]
fn detects_negated_if() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for `if !`, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/NegatedIf"));
}

#[test]
fn no_violation_with_unless() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = NegatedIf.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

use rubric_core::{LintContext, Rule};
use rubric_rules::lint::float_out_of_range::FloatOutOfRange;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/float_out_of_range/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/float_out_of_range/corrected.rb");

#[test]
fn detects_float_out_of_range() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/FloatOutOfRange"));
}

#[test]
fn no_violation_for_normal_float() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

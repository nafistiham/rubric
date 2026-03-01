use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_lines_around_method_body::EmptyLinesAroundMethodBody;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/empty_lines_around_method_body/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/empty_lines_around_method_body/corrected.rb");

#[test]
fn detects_empty_lines_around_method_body() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLinesAroundMethodBody.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLinesAroundMethodBody"));
}

#[test]
fn no_violation_for_clean_method_body() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyLinesAroundMethodBody.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

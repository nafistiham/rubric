use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_lines_around_module_body::EmptyLinesAroundModuleBody;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/empty_lines_around_module_body/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/empty_lines_around_module_body/corrected.rb");

#[test]
fn detects_empty_lines_around_module_body() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLinesAroundModuleBody.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLinesAroundModuleBody"));
}

#[test]
fn no_violation_for_clean_module_body() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyLinesAroundModuleBody.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

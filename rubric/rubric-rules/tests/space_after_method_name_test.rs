use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_after_method_name::SpaceAfterMethodName;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_after_method_name/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_after_method_name/corrected.rb");

#[test]
fn detects_space_after_method_name() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAfterMethodName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAfterMethodName"));
}

#[test]
fn no_violation_for_no_space_after_method_name() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAfterMethodName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

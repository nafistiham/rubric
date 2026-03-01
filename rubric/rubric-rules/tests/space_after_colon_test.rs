use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_after_colon::SpaceAfterColon;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_after_colon/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_after_colon/corrected.rb");

#[test]
fn detects_missing_space_after_colon() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAfterColon"));
}

#[test]
fn no_violation_with_space_after_colon() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

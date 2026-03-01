use rubric_core::{LintContext, Rule};
use rubric_rules::layout::leading_comment_space::LeadingCommentSpace;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/leading_comment_space/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/leading_comment_space/corrected.rb");

#[test]
fn detects_comment_without_space() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = LeadingCommentSpace.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/LeadingCommentSpace"));
}

#[test]
fn no_violation_with_space_after_hash() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = LeadingCommentSpace.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

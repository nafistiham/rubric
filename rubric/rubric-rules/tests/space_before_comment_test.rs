use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_before_comment::SpaceBeforeComment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_before_comment/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/space_before_comment/corrected.rb");

#[test]
fn detects_missing_space_before_comment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceBeforeComment.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceBeforeComment"));
}

#[test]
fn no_violation_with_space_before_comment() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceBeforeComment.check_source(&ctx);
    assert!(diags.is_empty());
}

use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_around_keyword::SpaceAroundKeyword;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_around_keyword/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_around_keyword/corrected.rb");

#[test]
fn detects_keyword_without_space() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAroundKeyword.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAroundKeyword"));
}

#[test]
fn no_violation_with_space_around_keyword() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAroundKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

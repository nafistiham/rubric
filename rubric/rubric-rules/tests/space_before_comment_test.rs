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

#[test]
fn no_false_positive_on_hash_in_url() {
    // A `#` inside a string (e.g. a URL) should not be flagged as a comment
    // because it is not followed by a space.
    let source = "url = \"http://example.com#anchor\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = SpaceBeforeComment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "# inside a URL string should not produce a violation, got: {:?}",
        diags
    );
}

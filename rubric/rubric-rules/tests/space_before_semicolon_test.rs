use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_before_semicolon::SpaceBeforeSemicolon;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_before_semicolon/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceBeforeSemicolon.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceBeforeSemicolon"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo;\nbar = 1; baz = 2\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceBeforeSemicolon.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

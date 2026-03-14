use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_before_comma::SpaceBeforeComma;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_before_comma/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/layout/space_before_comma/clean.rb");

#[test]
fn detects_space_before_comma() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceBeforeComma.check_source(&ctx);
    assert!(diags.len() >= 3, "expected at least 3 violations, got {}", diags.len());
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceBeforeComma"));
}

#[test]
fn no_violation_when_no_space_before_comma() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = SpaceBeforeComma.check_source(&ctx);
    assert_eq!(diags.len(), 0, "expected no violations");
}

#[test]
fn skips_comma_inside_string() {
    let source = "x = \"a ,b\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = SpaceBeforeComma.check_source(&ctx);
    assert_eq!(diags.len(), 0, "should not flag comma inside string");
}

#[test]
fn skips_comment_lines() {
    let source = "# foo ,bar\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = SpaceBeforeComma.check_source(&ctx);
    assert_eq!(diags.len(), 0, "should not flag comma inside comment");
}

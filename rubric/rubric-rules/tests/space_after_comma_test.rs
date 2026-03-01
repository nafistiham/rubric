use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_after_comma::SpaceAfterComma;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_after_comma/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/space_after_comma/corrected.rb");

#[test]
fn detects_missing_space_after_comma() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAfterComma"));
}

#[test]
fn no_violation_with_spaces_after_comma() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(diags.is_empty());
}

#[test]
fn no_false_positive_on_comma_in_string() {
    let source = "foo(\"a,b\", \"c,d\")\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(diags.is_empty(), "should not flag commas inside strings, got: {:?}", diags);
}

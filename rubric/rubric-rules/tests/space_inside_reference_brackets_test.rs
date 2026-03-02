use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_inside_reference_brackets::SpaceInsideReferenceBrackets;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_inside_reference_brackets/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceInsideReferenceBrackets.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceInsideReferenceBrackets"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo[1]\nbar[:key]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideReferenceBrackets.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_false_positive_for_multiline_reference_close() {
    let src = "result = foo([\n  Foo.bar,\n  ])\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideReferenceBrackets.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for multiline bracket close, got: {:?}", diags);
}

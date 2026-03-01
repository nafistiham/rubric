use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_around_equals_in_parameter_default::SpaceAroundEqualsInParameterDefault;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_around_equals_in_parameter_default/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAroundEqualsInParameterDefault.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAroundEqualsInParameterDefault"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo(bar = 1, baz = 2)\n  bar + baz\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundEqualsInParameterDefault.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

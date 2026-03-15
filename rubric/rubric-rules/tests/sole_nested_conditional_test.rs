use rubric_core::{LintContext, Rule};
use rubric_rules::style::sole_nested_conditional::SoleNestedConditional;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/sole_nested_conditional/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/sole_nested_conditional/clean.rb");

#[test]
fn detects_sole_nested_if() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SoleNestedConditional.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(
        diags
            .iter()
            .all(|d| d.rule == "Style/SoleNestedConditional"),
        "all diagnostics should be tagged correctly"
    );
}

#[test]
fn reports_correct_count_for_offending() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SoleNestedConditional.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations (one per outer if/unless)");
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = SoleNestedConditional.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_violation_when_outer_has_else() {
    let src = "if foo\n  if bar\n    do_something\n  end\nelse\n  other\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SoleNestedConditional.check_source(&ctx);
    assert!(diags.is_empty(), "outer if with else should not be flagged");
}

#[test]
fn no_violation_when_inner_has_else() {
    let src = "if foo\n  if bar\n    do_something\n  else\n    other\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SoleNestedConditional.check_source(&ctx);
    assert!(diags.is_empty(), "inner if with else should not be flagged");
}

#[test]
fn no_violation_when_outer_body_has_extra_statements() {
    let src = "if foo\n  bar\n  if baz\n    qux\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SoleNestedConditional.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "outer if with extra body statements should not be flagged"
    );
}

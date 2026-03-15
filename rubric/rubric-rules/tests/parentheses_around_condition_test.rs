use rubric_core::{LintContext, Rule};
use rubric_rules::style::parentheses_around_condition::ParenthesesAroundCondition;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/parentheses_around_condition/offending.rb");
const PASSING: &str =
    include_str!("fixtures/style/parentheses_around_condition/passing.rb");

#[test]
fn detects_if_with_parens() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/ParenthesesAroundCondition"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_while_with_parens() {
    let src = "while (running)\n  step\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "while (cond) should be flagged");
}

#[test]
fn detects_until_with_parens() {
    let src = "until (done)\n  wait\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "until (cond) should be flagged");
}

#[test]
fn detects_unless_with_parens() {
    let src = "unless (valid?)\n  raise\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "unless (cond) should be flagged");
}

#[test]
fn no_violation_for_method_call() {
    let src = "result = foo(bar)\nx = method_call(arg)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert!(diags.is_empty(), "method calls should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_plain_if() {
    let src = "if condition\n  do_something\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert!(diags.is_empty(), "plain if without parens should not be flagged");
}

#[test]
fn no_violation_for_comment() {
    let src = "# if (x)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged");
}

#[test]
fn detects_exactly_four_keywords() {
    // Verify one diag per keyword form in the offending fixture
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ParenthesesAroundCondition.check_source(&ctx);
    assert_eq!(diags.len(), 4, "expected 4 violations (if/while/until/unless), got: {:?}", diags);
}

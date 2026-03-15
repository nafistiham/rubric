use rubric_core::{LintContext, Rule};
use rubric_rules::lint::binary_operator_with_identical_operands::BinaryOperatorWithIdenticalOperands;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/binary_operator_with_identical_operands/offending.rb");
const PASSING: &str =
    include_str!("fixtures/lint/binary_operator_with_identical_operands/passing.rb");

#[test]
fn detects_violations_in_offending() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(
        diags.iter().all(|d| d.rule == "Lint/BinaryOperatorWithIdenticalOperands"),
        "unexpected rule names: {:?}",
        diags.iter().map(|d| d.rule).collect::<Vec<_>>()
    );
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_equality_with_same_identifier() {
    let src = "foo == foo\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `foo == foo`");
    assert!(diags[0].message.contains("=="));
}

#[test]
fn detects_logical_and_with_same_identifier() {
    let src = "x && x\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `x && x`");
}

#[test]
fn does_not_flag_different_operands() {
    let src = "a + b\nfoo == bar\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(diags.is_empty(), "different operands must not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# x + x\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(diags.is_empty(), "comment line must not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_string_containing_pattern() {
    let src = "msg = \"x == x\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(diags.is_empty(), "pattern inside string must not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_receiver_dot_method() {
    // `a.foo == b.foo` — different receivers, different expressions
    let src = "a.foo == b.foo\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(diags.is_empty(), "receiver.method must not be flagged: {:?}", diags);
}

#[test]
fn message_includes_operator() {
    let src = "x - x\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorWithIdenticalOperands.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
    assert!(diags[0].message.contains('-'), "message should mention operator `-`");
}

use rubric_core::{LintContext, Rule};
use rubric_rules::naming::binary_operator_parameter_name::BinaryOperatorParameterName;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/naming/binary_operator_parameter_name/offending.rb");
const CLEAN: &str = include_str!("fixtures/naming/binary_operator_parameter_name/clean.rb");

#[test]
fn detects_plus_with_wrong_param() {
    let src = "def +(value)\n  @val + value\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorParameterName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for def +(value)");
    assert_eq!(diags[0].rule, "Naming/BinaryOperatorParameterName");
    assert!(diags[0].message.contains('+'));
}

#[test]
fn detects_eq_eq_with_wrong_param() {
    let src = "def ==(rhs)\n  @val == rhs\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorParameterName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for def ==(rhs)");
    assert!(diags[0].message.contains("=="));
}

#[test]
fn no_violation_when_named_other() {
    let src = "def +(other)\n  @val + other\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorParameterName.check_source(&ctx);
    assert!(diags.is_empty(), "def +(other) should not be flagged");
}

#[test]
fn no_violation_eq_eq_other() {
    let src = "def ==(other)\n  @val == other\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorParameterName.check_source(&ctx);
    assert!(diags.is_empty(), "def ==(other) should not be flagged");
}

#[test]
fn detects_spaceship_with_wrong_param() {
    let src = "def <=>(x)\n  @val <=> x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorParameterName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for def <=>(x)");
    assert!(diags[0].message.contains("<=>"));
}

#[test]
fn no_violation_in_comment() {
    let src = "# def +(value) is wrong\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BinaryOperatorParameterName.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BinaryOperatorParameterName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags
        .iter()
        .all(|d| d.rule == "Naming/BinaryOperatorParameterName"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = BinaryOperatorParameterName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb, got: {:?}", diags);
}

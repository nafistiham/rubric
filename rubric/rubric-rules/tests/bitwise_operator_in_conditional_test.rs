use rubric_core::{LintContext, Rule};
use rubric_rules::style::bitwise_operator_in_conditional::BitwiseOperatorInConditional;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/bitwise_operator_in_conditional/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/bitwise_operator_in_conditional/clean.rb");

#[test]
fn detects_bitwise_and_in_if() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/BitwiseOperatorInConditional"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_single_ampersand_in_if() {
    let src = "if foo & bar\n  x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(!diags.is_empty(), "single & in if should be flagged");
    assert!(
        diags[0].message.contains("bitwise"),
        "message should mention bitwise operator"
    );
}

#[test]
fn flags_single_pipe_in_while() {
    let src = "while a | b\n  loop_body\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(!diags.is_empty(), "single | in while should be flagged");
}

#[test]
fn does_not_flag_double_ampersand() {
    let src = "if foo && bar\n  x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(diags.is_empty(), "&& should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_double_pipe() {
    let src = "if foo || bar\n  x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(diags.is_empty(), "|| should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_safe_navigation() {
    let src = "if obj&.method\n  x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(diags.is_empty(), "&. safe navigation should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# if a & b then do something\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_unless_with_bitwise() {
    let src = "unless x | y\n  skip\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BitwiseOperatorInConditional.check_source(&ctx);
    assert!(!diags.is_empty(), "bitwise | in unless should be flagged");
}

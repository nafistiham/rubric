use rubric_core::{LintContext, Rule};
use rubric_rules::lint::literal_as_condition::LiteralAsCondition;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/literal_as_condition/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/lint/literal_as_condition/clean.rb");

#[test]
fn detects_literal_conditions() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/LiteralAsCondition"));
}

#[test]
fn no_violation_for_variable_conditions() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_if_nil() {
    let src = "if nil\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "if nil should be flagged");
    assert!(diags[0].message.contains("nil"));
}

#[test]
fn flags_if_true() {
    let src = "if true\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "if true should be flagged");
}

#[test]
fn flags_if_false() {
    let src = "if false\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "if false should be flagged");
}

#[test]
fn flags_if_integer() {
    let src = "if 42\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "if 42 should be flagged");
    assert!(diags[0].message.contains("42"));
}

#[test]
fn flags_if_string() {
    let src = "if \"hello\"\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "if string literal should be flagged");
}

#[test]
fn flags_unless_nil() {
    let src = "unless nil\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "unless nil should be flagged");
}

#[test]
fn flags_while_true() {
    let src = "while true\n  break\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "while true should be flagged");
}

#[test]
fn flags_until_false() {
    let src = "until false\n  break\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "until false should be flagged");
}

#[test]
fn does_not_flag_variable_condition() {
    let src = "if some_var\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(diags.is_empty(), "variable condition should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# if nil\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(diags.is_empty(), "comment line should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_if_symbol() {
    let src = "if :sym\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralAsCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "if :sym should be flagged");
}

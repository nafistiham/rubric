use rubric_core::{LintContext, Rule};
use rubric_rules::style::character_literal::CharacterLiteral;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/character_literal/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/character_literal/clean.rb");

#[test]
fn detects_character_literals() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CharacterLiteral.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/CharacterLiteral"));
}

#[test]
fn no_violation_for_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = CharacterLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn does_not_flag_method_ending_question_mark() {
    let src = "def valid?\n  true\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CharacterLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "method ending ? should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_question_mark_in_comment() {
    let src = "# use ?a for char literal\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CharacterLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "?a in comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_in_string() {
    let src = "x = \"?a\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CharacterLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "?a in string should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_char_literal_with_message() {
    let src = "x = ?a\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CharacterLiteral.check_source(&ctx);
    assert!(!diags.is_empty(), "?a should be flagged");
    assert!(diags[0].message.contains("string literal"), "message should mention string literal");
}

#[test]
fn counts_correct_number_of_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CharacterLiteral.check_source(&ctx);
    assert_eq!(diags.len(), 3, "expected 3 violations, got {}", diags.len());
}

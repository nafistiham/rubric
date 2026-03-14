use rubric_core::{LintContext, Rule};
use rubric_rules::style::string_chars::StringChars;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/string_chars/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/string_chars/clean.rb");

#[test]
fn detects_split_with_empty_string_or_regex() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = StringChars.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/StringChars"));
}

#[test]
fn no_violation_for_chars_calls() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = StringChars.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_split_with_double_quoted_empty_string() {
    let src = "\"hello\".split(\"\")\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StringChars.check_source(&ctx);
    assert!(!diags.is_empty(), "split(\"\") should be flagged");
}

#[test]
fn flags_split_with_single_quoted_empty_string() {
    let src = "str.split('')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StringChars.check_source(&ctx);
    assert!(!diags.is_empty(), "split('') should be flagged");
}

#[test]
fn flags_split_with_empty_regex() {
    let src = "text.split(//)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StringChars.check_source(&ctx);
    assert!(!diags.is_empty(), "split(//) should be flagged");
}

#[test]
fn does_not_flag_split_with_non_empty_argument() {
    let src = "str.split(', ')\nstr.split(/,/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StringChars.check_source(&ctx);
    assert!(diags.is_empty(), "split with real separator should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_split_in_comment() {
    let src = "# str.split('')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StringChars.check_source(&ctx);
    assert!(diags.is_empty(), "split in comment should not be flagged, got: {:?}", diags);
}

#[test]
fn violation_message_mentions_chars() {
    let src = "str.split('')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StringChars.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("chars"),
        "message should mention chars, got: {}",
        diags[0].message
    );
}

#[test]
fn counts_all_three_offending_patterns() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = StringChars.check_source(&ctx);
    assert_eq!(diags.len(), 3, "expected 3 violations (one per line), got: {:?}", diags);
}

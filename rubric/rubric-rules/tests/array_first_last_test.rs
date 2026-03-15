use rubric_core::{LintContext, Rule};
use rubric_rules::style::array_first_last::ArrayFirstLast;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/array_first_last/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/array_first_last/clean.rb");

#[test]
fn detects_array_first_last() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ArrayFirstLast.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/ArrayFirstLast"));
}

#[test]
fn no_violation_for_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = ArrayFirstLast.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn does_not_flag_array_literal() {
    let src = "arr = [0, 1, 2]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayFirstLast.check_source(&ctx);
    assert!(diags.is_empty(), "[0] as array literal should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_other_indices() {
    let src = "arr[1]\narr[-2]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayFirstLast.check_source(&ctx);
    assert!(diags.is_empty(), "arr[1] and arr[-2] should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_zero_index_with_message() {
    let src = "arr[0]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayFirstLast.check_source(&ctx);
    assert!(!diags.is_empty(), "arr[0] should be flagged");
    assert!(diags[0].message.contains("first"), "message should mention 'first'");
}

#[test]
fn flags_minus_one_index_with_message() {
    let src = "arr[-1]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayFirstLast.check_source(&ctx);
    assert!(!diags.is_empty(), "arr[-1] should be flagged");
    assert!(diags[0].message.contains("last"), "message should mention 'last'");
}

#[test]
fn does_not_flag_in_comment() {
    let src = "# arr[0] is the first element\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayFirstLast.check_source(&ctx);
    assert!(diags.is_empty(), "arr[0] in comment should not be flagged, got: {:?}", diags);
}

#[test]
fn counts_correct_number_of_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ArrayFirstLast.check_source(&ctx);
    assert_eq!(diags.len(), 3, "expected 3 violations, got {}", diags.len());
}

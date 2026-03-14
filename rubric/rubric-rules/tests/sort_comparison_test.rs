use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/style/sort_comparison.rs"]
mod sort_comparison;
use sort_comparison::SortComparison;

const OFFENDING: &str = include_str!("fixtures/style/sort_comparison/offending.rb");
const PASSING: &str = include_str!("fixtures/style/sort_comparison/passing.rb");

#[test]
fn detects_trivial_sort_block() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SortComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/SortComparison"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = SortComparison.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_sort_with_spaceship_ascending() {
    let src = "arr.sort { |a, b| a <=> b }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SortComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
    assert!(
        diags[0].message.contains("sort"),
        "message should mention sort, got: {}",
        diags[0].message
    );
}

#[test]
fn does_not_flag_descending_sort() {
    let src = "arr.sort { |a, b| b <=> a }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SortComparison.check_source(&ctx);
    assert!(diags.is_empty(), "descending sort should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_sort_by() {
    let src = "arr.sort_by { |a| a.name }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SortComparison.check_source(&ctx);
    assert!(diags.is_empty(), "sort_by should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_bare_sort() {
    let src = "arr.sort\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SortComparison.check_source(&ctx);
    assert!(diags.is_empty(), "bare sort should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# arr.sort { |a, b| a <=> b }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SortComparison.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

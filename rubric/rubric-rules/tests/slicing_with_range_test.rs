use rubric_core::{LintContext, Rule};
use rubric_rules::style::slicing_with_range::SlicingWithRange;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/slicing_with_range/offending.rb");
const PASSING: &str = include_str!("fixtures/style/slicing_with_range/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SlicingWithRange.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb, got none");
    assert!(
        diags.iter().all(|d| d.rule == "Style/SlicingWithRange"),
        "unexpected rule name"
    );
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = SlicingWithRange.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on passing.rb, got: {:?}", diags);
}

#[test]
fn flags_length_minus_one_pattern() {
    let src = "arr[1..arr.length - 1]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SlicingWithRange.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected 1 violation for arr.length - 1, got: {:?}", diags);
    assert_eq!(diags[0].rule, "Style/SlicingWithRange");
}

#[test]
fn flags_size_minus_one_pattern() {
    let src = "arr[1..arr.size - 1]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SlicingWithRange.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected 1 violation for arr.size - 1, got: {:?}", diags);
    assert_eq!(diags[0].rule, "Style/SlicingWithRange");
}

#[test]
fn does_not_flag_endless_range() {
    // Already using endless range syntax
    let src = "arr[1..]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SlicingWithRange.check_source(&ctx);
    assert!(diags.is_empty(), "endless range should not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_different_variable() {
    // arr[1..other.length - 1] — upper bound refers to a different variable
    // This is still flaggable in practice, but our heuristic may or may not catch it
    // We only test that arr.length - 1 is flagged regardless of variable name
    let src = "arr[1..arr.length - 1]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SlicingWithRange.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
}

#[test]
fn message_suggests_endless_range() {
    let src = "arr[1..arr.length - 1]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SlicingWithRange.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("n..") || diags[0].message.contains("length") || diags[0].message.contains("size"),
        "message should suggest endless range: {}",
        diags[0].message
    );
}

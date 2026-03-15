use rubric_core::{LintContext, Rule};
use rubric_rules::style::collection_compact::CollectionCompact;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/collection_compact/offending.rb");
const CORRECTED: &str = include_str!("fixtures/style/collection_compact/corrected.rb");

#[test]
fn detects_nil_filtering_patterns() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for nil-filter patterns, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/CollectionCompact"));
}

#[test]
fn no_violation_for_compact() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for `.compact`, got: {:?}", diags);
}

#[test]
fn detects_select_not_nil() {
    let src = "result = arr.select { |x| !x.nil? }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(!diags.is_empty(), "`.select {{ |x| !x.nil? }}` should be flagged");
    assert_eq!(diags[0].rule, "Style/CollectionCompact");
}

#[test]
fn detects_reject_nil() {
    let src = "result = arr.reject { |x| x.nil? }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(!diags.is_empty(), "`.reject {{ |x| x.nil? }}` should be flagged");
}

#[test]
fn detects_filter_not_nil() {
    let src = "result = arr.filter { |item| !item.nil? }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(!diags.is_empty(), "`.filter {{ |item| !item.nil? }}` should be flagged");
}

#[test]
fn detects_select_not_equal_nil() {
    let src = "result = arr.select { |x| x != nil }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(!diags.is_empty(), "`.select {{ |x| x != nil }}` should be flagged");
}

#[test]
fn detects_reject_equal_nil() {
    let src = "result = arr.reject { |x| x == nil }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(!diags.is_empty(), "`.reject {{ |x| x == nil }}` should be flagged");
}

#[test]
fn no_violation_for_select_with_other_condition() {
    // filtering by something other than nil is fine
    let src = "result = arr.select { |x| x > 0 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(diags.is_empty(), "`.select` with non-nil condition should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_reject_with_other_condition() {
    let src = "result = arr.reject { |x| x.empty? }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(diags.is_empty(), "`.reject` with non-nil condition should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_comment_line() {
    let src = "# arr.select { |x| !x.nil? }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionCompact.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}

#[test]
fn counts_all_violations_in_offending_file() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CollectionCompact.check_source(&ctx);
    // offending.rb has 5 violation lines
    assert_eq!(diags.len(), 5, "expected 5 violations in offending.rb, got {}", diags.len());
}

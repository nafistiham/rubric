use rubric_core::{LintContext, Rule};
use rubric_rules::style::collection_methods::CollectionMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/collection_methods/offending.rb");
const PASSING: &str = include_str!("fixtures/style/collection_methods/passing.rb");

#[test]
fn detects_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/CollectionMethods"));
}

#[test]
fn no_violation_on_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}

#[test]
fn flags_collect_with_correct_message() {
    let src = "[1,2,3].collect { |x| x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for .collect");
    assert!(diags[0].message.contains("map"), "message should mention map");
}

#[test]
fn flags_inject() {
    let src = "[1,2,3].inject(0) { |sum, x| sum + x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for .inject");
    assert!(diags[0].message.contains("reduce"));
}

#[test]
fn does_not_flag_word_with_prefix() {
    // "injected" should not be flagged (inject is a prefix of injected)
    let src = "x = obj.injected\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(diags.is_empty(), "injected should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_in_comment() {
    let src = "# arr.collect { |x| x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_detect() {
    let src = "arr.detect { |x| x > 0 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for .detect");
    assert!(diags[0].message.contains("find"));
}

#[test]
fn flags_find_all() {
    let src = "arr.find_all { |x| x > 0 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for .find_all");
    assert!(diags[0].message.contains("select"));
}

#[test]
fn flags_collect_concat() {
    let src = "arr.collect_concat { |x| x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CollectionMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for .collect_concat");
    assert!(diags[0].message.contains("flat_map"));
}

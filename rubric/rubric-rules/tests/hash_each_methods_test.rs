use rubric_core::{LintContext, Rule};
use rubric_rules::style::hash_each_methods::HashEachMethods;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/hash_each_methods/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/hash_each_methods/clean.rb");

#[test]
fn detects_keys_each() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = HashEachMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/HashEachMethods"));
}

#[test]
fn no_violation_for_clean_hash_methods() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = HashEachMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_keys_each_with_message() {
    let src = "hash.keys.each { |k| puts k }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashEachMethods.check_source(&ctx);
    assert!(!diags.is_empty(), ".keys.each should be flagged");
    assert!(diags[0].message.contains("each_key"));
}

#[test]
fn flags_values_each_with_message() {
    let src = "hash.values.each { |v| puts v }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashEachMethods.check_source(&ctx);
    assert!(!diags.is_empty(), ".values.each should be flagged");
    assert!(diags[0].message.contains("each_value"));
}

#[test]
fn does_not_flag_each_key() {
    let src = "hash.each_key { |k| puts k }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashEachMethods.check_source(&ctx);
    assert!(diags.is_empty(), "each_key should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_each_value() {
    let src = "hash.each_value { |v| puts v }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashEachMethods.check_source(&ctx);
    assert!(diags.is_empty(), "each_value should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# hash.keys.each { |k| }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashEachMethods.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_in_string() {
    let src = "puts \"hash.keys.each\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashEachMethods.check_source(&ctx);
    assert!(diags.is_empty(), "pattern inside string should not be flagged, got: {:?}", diags);
}

use rubric_core::{LintContext, Rule};
use rubric_rules::lint::each_with_object_argument::EachWithObjectArgument;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/each_with_object_argument/offending.rb");
const CLEAN: &str = include_str!("fixtures/lint/each_with_object_argument/clean.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/EachWithObjectArgument"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_integer_argument() {
    let src = "[1, 2].each_with_object(0) { |x, memo| memo + x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "integer argument should be flagged");
    assert_eq!(diags[0].rule, "Lint/EachWithObjectArgument");
    assert!(diags[0].message.contains("immutable"));
}

#[test]
fn detects_symbol_argument() {
    let src = "[:a, :b].each_with_object(:init) { |x, memo| x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "symbol argument should be flagged");
}

#[test]
fn detects_true_argument() {
    let src = "items.each_with_object(true) { |x, m| m }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "true should be flagged");
}

#[test]
fn detects_false_argument() {
    let src = "items.each_with_object(false) { |x, m| m }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "false should be flagged");
}

#[test]
fn detects_nil_argument() {
    let src = "items.each_with_object(nil) { |x, m| m }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "nil should be flagged");
}

#[test]
fn no_violation_on_array_argument() {
    let src = "[1, 2].each_with_object([]) { |x, memo| memo << x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(diags.is_empty(), "array argument should not be flagged");
}

#[test]
fn no_violation_on_hash_argument() {
    let src = "[1, 2].each_with_object({}) { |x, memo| memo[x] = x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(diags.is_empty(), "hash argument should not be flagged");
}

#[test]
fn does_not_flag_comment() {
    let src = "# [1].each_with_object(0) { |x, m| m }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EachWithObjectArgument.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

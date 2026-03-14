use rubric_core::{LintContext, Rule};
use rubric_rules::lint::literal_in_interpolation::LiteralInInterpolation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/literal_in_interpolation/offending.rb");
const CLEAN: &str = include_str!("fixtures/lint/literal_in_interpolation/clean.rb");

#[test]
fn detects_integer_in_interpolation() {
    let src = "\"Value is #{42}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for #{{42}}");
    assert!(diags[0].message.contains("Literal interpolation"));
}

#[test]
fn detects_nil_in_interpolation() {
    let src = "\"Result: #{nil}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for #{{nil}}");
}

#[test]
fn detects_true_in_interpolation() {
    let src = "\"Flag: #{true}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for #{{true}}");
}

#[test]
fn detects_false_in_interpolation() {
    let src = "\"Flag: #{false}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for #{{false}}");
}

#[test]
fn detects_symbol_in_interpolation() {
    let src = "\"Key: #{:foo}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for #{{:foo}}");
}

#[test]
fn no_violation_for_variable_interpolation() {
    let src = "\"Value is #{x}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "variable interpolation should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_method_call_interpolation() {
    let src = "\"Result: #{compute()}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "method call should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_in_single_quoted_string() {
    let src = "'Value is #{42}'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "single-quoted string should not be flagged, got: {:?}", diags);
}

#[test]
fn skips_comment_lines() {
    let src = "# \"#{42}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn rule_name_is_correct() {
    assert_eq!(LiteralInInterpolation.name(), "Lint/LiteralInInterpolation");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/LiteralInInterpolation"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = LiteralInInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb, got: {:?}", diags);
}

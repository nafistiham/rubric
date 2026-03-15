use rubric_core::{LintContext, Rule};
use rubric_rules::style::trailing_body_on_method_definition::TrailingBodyOnMethodDefinition;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/trailing_body_on_method_definition/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/trailing_body_on_method_definition/clean.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TrailingBodyOnMethodDefinition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(
        diags
            .iter()
            .all(|d| d.rule == "Style/TrailingBodyOnMethodDefinition"),
        "all diagnostics should be tagged correctly"
    );
}

#[test]
fn detects_correct_count() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TrailingBodyOnMethodDefinition.check_source(&ctx);
    assert_eq!(diags.len(), 3, "expected 3 violations");
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = TrailingBodyOnMethodDefinition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_violation_empty_method() {
    let src = "def foo; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrailingBodyOnMethodDefinition.check_source(&ctx);
    assert!(diags.is_empty(), "empty single-line method should not be flagged");
}

#[test]
fn detects_method_with_params_and_body() {
    let src = "def multiply(a, b); a * b; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrailingBodyOnMethodDefinition.check_source(&ctx);
    assert_eq!(diags.len(), 1, "method with params and body on same line should be flagged");
}

#[test]
fn no_violation_multiline_method() {
    let src = "def foo\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrailingBodyOnMethodDefinition.check_source(&ctx);
    assert!(diags.is_empty(), "multiline method should not be flagged");
}

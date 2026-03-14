use rubric_core::{LintContext, Rule};
use rubric_rules::naming::constant_name::ConstantName;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/naming/constant_name/offending.rb");
const CLEAN: &str = include_str!("fixtures/naming/constant_name/clean.rb");

#[test]
fn detects_mixed_case_constant() {
    let src = "My_Constant = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for My_Constant");
    assert!(diags[0].message.contains("SCREAMING_SNAKE_CASE"));
}

#[test]
fn no_violation_screaming_snake_case() {
    let src = "MAX_SIZE = 100\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantName.check_source(&ctx);
    assert!(diags.is_empty(), "SCREAMING_SNAKE_CASE should not be flagged");
}

#[test]
fn no_violation_class_style_constant() {
    let src = "FooBar = Class.new\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantName.check_source(&ctx);
    assert!(diags.is_empty(), "CamelCase class constant should not be flagged");
}

#[test]
fn no_violation_class_definition() {
    let src = "class MyClass\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantName.check_source(&ctx);
    assert!(diags.is_empty(), "class definition should not be flagged");
}

#[test]
fn no_violation_module_definition() {
    let src = "module MyModule\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantName.check_source(&ctx);
    assert!(diags.is_empty(), "module definition should not be flagged");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ConstantName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Naming/ConstantName"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = ConstantName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb");
}

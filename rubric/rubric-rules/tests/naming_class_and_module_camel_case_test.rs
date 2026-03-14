use rubric_core::{LintContext, Rule};
use rubric_rules::naming::class_and_module_camel_case::ClassAndModuleCamelCase;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/naming/class_and_module_camel_case/offending.rb");
const CLEAN: &str = include_str!("fixtures/naming/class_and_module_camel_case/clean.rb");

#[test]
fn detects_snake_case_class_name() {
    let src = "class my_class; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleCamelCase.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for snake_case class name");
    assert!(diags[0].message.contains("CamelCase"));
}

#[test]
fn detects_mixed_underscore_module_name() {
    let src = "module My_Module; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleCamelCase.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for My_Module");
    assert!(diags[0].message.contains("CamelCase"));
}

#[test]
fn no_violation_for_camel_case_class() {
    let src = "class MyClass; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleCamelCase.check_source(&ctx);
    assert!(diags.is_empty(), "CamelCase class should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_camel_case_module() {
    let src = "module MyModule; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleCamelCase.check_source(&ctx);
    assert!(diags.is_empty(), "CamelCase module should not be flagged, got: {:?}", diags);
}

#[test]
fn skips_class_singleton_self() {
    let src = "class << self; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleCamelCase.check_source(&ctx);
    assert!(diags.is_empty(), "`class << self` should not be flagged, got: {:?}", diags);
}

#[test]
fn skips_comment_lines() {
    let src = "# class my_class\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleCamelCase.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn rule_name_is_correct() {
    assert_eq!(ClassAndModuleCamelCase.name(), "Naming/ClassAndModuleCamelCase");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClassAndModuleCamelCase.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Naming/ClassAndModuleCamelCase"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = ClassAndModuleCamelCase.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb, got: {:?}", diags);
}

use rubric_core::{LintContext, Rule};
use rubric_rules::naming::variable_name::VariableName;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/naming/variable_name/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/naming/variable_name/clean.rb");

#[test]
fn detects_camel_case_variables() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = VariableName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Naming/VariableName"));
}

#[test]
fn no_violation_for_snake_case_variables() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = VariableName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_camel_case_with_message() {
    let src = "myVar = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = VariableName.check_source(&ctx);
    assert!(!diags.is_empty(), "camelCase var should be flagged");
    assert!(diags[0].message.contains("snake_case"));
}

#[test]
fn does_not_flag_snake_case() {
    let src = "my_var = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = VariableName.check_source(&ctx);
    assert!(diags.is_empty(), "snake_case var should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_constant() {
    let src = "CONSTANT = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = VariableName.check_source(&ctx);
    assert!(diags.is_empty(), "constants should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_instance_variable() {
    let src = "@myVar = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = VariableName.check_source(&ctx);
    assert!(diags.is_empty(), "instance variables should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_class_variable() {
    let src = "@@myVar = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = VariableName.check_source(&ctx);
    assert!(diags.is_empty(), "class variables should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_global_variable() {
    let src = "$myVar = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = VariableName.check_source(&ctx);
    assert!(diags.is_empty(), "global variables should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_equality_check() {
    let src = "if myVar == 42\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = VariableName.check_source(&ctx);
    assert!(diags.is_empty(), "== should not trigger variable detection, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# myVar = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = VariableName.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}

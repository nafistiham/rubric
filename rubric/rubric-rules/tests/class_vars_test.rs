use rubric_core::{LintContext, Rule};
use rubric_rules::style::class_vars::ClassVars;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/class_vars/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/class_vars/clean.rb");

#[test]
fn detects_class_variables() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClassVars.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations (@@count assignment and usage)");
    assert!(diags.iter().all(|d| d.rule == "Style/ClassVars"));
}

#[test]
fn no_violation_for_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = ClassVars.check_source(&ctx);
    assert_eq!(diags.len(), 0, "expected no violations for clean code");
}

#[test]
fn skips_instance_variables() {
    let source = "@count = 0\n@name = 'foo'\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = ClassVars.check_source(&ctx);
    assert_eq!(diags.len(), 0, "single @ instance variables should not be flagged");
}

#[test]
fn skips_class_var_inside_string() {
    let source = "x = \"@@var is bad\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = ClassVars.check_source(&ctx);
    assert_eq!(diags.len(), 0, "@@var inside string should not be flagged");
}

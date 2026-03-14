use rubric_core::{LintContext, Rule};
use rubric_rules::naming::accessor_method_name::AccessorMethodName;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/naming/accessor_method_name/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/naming/accessor_method_name/clean.rb");

#[test]
fn detects_get_prefix() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AccessorMethodName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Naming/AccessorMethodName"));
}

#[test]
fn no_violation_for_clean_names() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = AccessorMethodName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_get_prefix() {
    let src = "def get_name\n  @name\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = AccessorMethodName.check_source(&ctx);
    assert!(!diags.is_empty(), "get_ prefix should be flagged");
    assert!(diags[0].message.contains("get_"));
}

#[test]
fn flags_set_prefix() {
    let src = "def set_name(val)\n  @name = val\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = AccessorMethodName.check_source(&ctx);
    assert!(!diags.is_empty(), "set_ prefix should be flagged");
    assert!(diags[0].message.contains("set_"));
}

#[test]
fn does_not_flag_get_without_underscore() {
    let src = "def getter\n  @x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = AccessorMethodName.check_source(&ctx);
    assert!(diags.is_empty(), "getter (no underscore) should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_set_exactly() {
    let src = "def set\n  @x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = AccessorMethodName.check_source(&ctx);
    assert!(diags.is_empty(), "set (no underscore suffix) should not be flagged, got: {:?}", diags);
}

use rubric_core::{LintContext, Rule};
use rubric_rules::naming::predicate_name::PredicateName;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/naming/predicate_name/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/naming/predicate_name/clean.rb");

#[test]
fn detects_forbidden_predicate_prefixes() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = PredicateName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Naming/PredicateName"));
}

#[test]
fn no_violation_for_clean_predicate_names() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = PredicateName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_is_prefix() {
    let src = "def is_valid?\n  true\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PredicateName.check_source(&ctx);
    assert!(!diags.is_empty(), "is_ prefix should be flagged");
    assert!(diags[0].message.contains("is_valid?"));
}

#[test]
fn flags_has_prefix() {
    let src = "def has_children?\n  false\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PredicateName.check_source(&ctx);
    assert!(!diags.is_empty(), "has_ prefix should be flagged");
}

#[test]
fn flags_have_prefix() {
    let src = "def have_access?\n  true\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PredicateName.check_source(&ctx);
    assert!(!diags.is_empty(), "have_ prefix should be flagged");
}

#[test]
fn does_not_flag_non_predicate_is_method() {
    let src = "def is_valid\n  true\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PredicateName.check_source(&ctx);
    assert!(diags.is_empty(), "is_valid (no ?) should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_clean_predicate() {
    let src = "def valid?\n  true\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PredicateName.check_source(&ctx);
    assert!(diags.is_empty(), "valid? should not be flagged, got: {:?}", diags);
}

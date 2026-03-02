use rubric_core::{LintContext, Rule};
use rubric_rules::lint::unused_method_argument::UnusedMethodArgument;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/unused_method_argument/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/unused_method_argument/corrected.rb");

#[test]
fn detects_unused_method_argument() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnusedMethodArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for unused arg, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnusedMethodArgument"));
}

#[test]
fn no_violation_with_all_args_used_or_prefixed() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UnusedMethodArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_keyword_arg_with_default_used_in_body() {
    // Keyword args like `name: nil` must have name extracted as `name` not `name: nil`
    let src = "def email(name: nil, domain: nil)\n  [name, domain].join('@')\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnusedMethodArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for keyword args used in body, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_required_keyword_arg_used_in_body() {
    // Required keyword arg `name:` (no default) should also be handled
    let src = "def greet(name:)\n  name.upcase\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnusedMethodArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for required keyword arg used in body, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_keyword_arg_used_in_string_interpolation() {
    // Keyword arg used inside "#{name}" string interpolation
    let src = "def email(name: nil, domain: nil)\n  \"#{name}@#{domain}\"\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnusedMethodArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for keyword arg used in string interpolation, got: {:?}", diags);
}

#[test]
fn still_detects_actually_unused_keyword_arg() {
    // Keyword arg that is genuinely not used in the body should still be flagged
    let src = "def industry(category: nil)\n  fetch('company.industry')\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnusedMethodArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for unused keyword arg, got none");
}

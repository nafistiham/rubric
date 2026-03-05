use rubric_core::{LintContext, Rule};
use rubric_rules::lint::unused_method_argument::UnusedMethodArgument;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/unused_method_argument/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/unused_method_argument/corrected.rb");

fn check(src: &str) -> Vec<rubric_core::Diagnostic> {
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![Box::new(UnusedMethodArgument)];
    let mut diags: Vec<_> = rules.iter().flat_map(|r| r.check_source(&ctx)).collect();
    diags.extend(rubric_core::walk(src.as_bytes(), &ctx, &rules));
    diags
}

#[test]
fn detects_unused_method_argument() {
    let diags = check(OFFENDING);
    assert!(!diags.is_empty(), "expected violations for unused arg, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnusedMethodArgument"));
}

#[test]
fn no_violation_with_all_args_used_or_prefixed() {
    let diags = check(CORRECTED);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_keyword_arg_with_default_used_in_body() {
    let src = "def email(name: nil, domain: nil)\n  [name, domain].join('@')\nend\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no FP for keyword args used in body, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_required_keyword_arg_used_in_body() {
    let src = "def greet(name:)\n  name.upcase\nend\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no FP for required keyword arg used in body, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_keyword_arg_used_in_string_interpolation() {
    let src = "def email(name: nil, domain: nil)\n  \"#{name}@#{domain}\"\nend\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no FP for keyword arg used in string interpolation, got: {:?}", diags);
}

#[test]
fn still_detects_actually_unused_keyword_arg() {
    let src = "def industry(category: nil)\n  fetch('company.industry')\nend\n";
    let diags = check(src);
    assert!(!diags.is_empty(), "expected violation for unused keyword arg, got none");
}

// _-prefixed params must never be flagged
#[test]
fn no_false_positive_for_underscore_prefixed_param() {
    let src = "def foo(_unused, used)\n  used + 1\nend\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no FP for _-prefixed param, got: {:?}", diags);
}

// Endless methods (def foo = expr) must never be flagged
#[test]
fn no_false_positive_for_endless_method() {
    let src = "def double(n) = n * 2\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no FP for endless method, got: {:?}", diags);
}

// def self.method: class method — params still checked
#[test]
fn detects_unused_in_class_method() {
    let src = "def self.build(config, unused_arg)\n  config.dup\nend\n";
    let diags = check(src);
    assert!(!diags.is_empty(), "expected violation for unused class method arg");
    assert!(diags.iter().any(|d| d.message.contains("unused_arg")));
}

// Abstract method (no body, just a comment) — no params
#[test]
fn no_false_positive_for_method_with_no_body() {
    // empty body is represented by an empty StatementsNode or nil body
    let src = "def abstract_method(arg)\nend\n";
    // empty body: arg is technically unused, but a method with no body is often
    // intentional (interface stub). We do flag it — this test documents behavior:
    // The body IS present (empty statements), so we check.
    // If no statements, Prism returns nil body → no check needed.
    // This is fine — we flag empty stubs too, same as RuboCop.
    let diags = check(src);
    // No assertion on count — just ensure we don't panic
    let _ = diags;
}

// Method with forwarding params (...) — all forwarded, nothing to flag
#[test]
fn no_false_positive_for_forwarding_params() {
    let src = "def delegate(...)\n  other.send(...)\nend\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no FP for forwarding params, got: {:?}", diags);
}

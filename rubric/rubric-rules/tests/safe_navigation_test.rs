use rubric_core::{LintContext, Rule};
use rubric_rules::style::safe_navigation::SafeNavigation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/safe_navigation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/safe_navigation/corrected.rb");

#[test]
fn detects_safe_navigation_opportunity() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for `x && x.foo` pattern, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/SafeNavigation"));
}

#[test]
fn no_violation_with_safe_navigation_operator() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// FP: `workers_env && workers_env.strip != ""` — trailing comparison changes semantics
#[test]
fn no_false_positive_for_var_and_var_method_with_trailing_comparison() {
    let src = "if workers_env && workers_env.strip != \"\"\n  setup\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for `var && var.method != x`, got: {:?}", diags);
}

// FP: `x && x.foo == bar` — equality comparison after method call
#[test]
fn no_false_positive_for_var_and_var_method_with_equality_comparison() {
    let src = "result = x && x.status == :active\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for `x && x.status == :active`, got: {:?}", diags);
}

// FP: `x && x.foo && bar` — additional `&&` after method call
#[test]
fn no_false_positive_for_var_and_var_method_with_trailing_and() {
    let src = "ok = x && x.valid? && x.active?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for `x && x.valid? && x.active?`, got: {:?}", diags);
}

// Real violation: simple `var && var.method` should still be flagged
#[test]
fn still_detects_simple_var_and_var_method() {
    let src = "result = user && user.name\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `user && user.name`, got none");
}

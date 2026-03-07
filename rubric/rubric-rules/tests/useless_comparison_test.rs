use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_comparison::UselessComparison;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/useless_comparison/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/useless_comparison/corrected.rb");

#[test]
fn detects_useless_comparison() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessComparison"));
}

#[test]
fn no_violation_for_different_operands() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UselessComparison.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// `thread.account_id == account_id` — LHS has a receiver (`thread.`), RHS is bare
// These are DIFFERENT expressions and must not be flagged
#[test]
fn no_false_positive_for_receiver_dot_attr_vs_bare_local() {
    let src = "if thread.account_id == account_id && thread.reply?\n  true\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessComparison.check_source(&ctx);
    assert!(diags.is_empty(), "receiver.attr == bare_local falsely flagged: {:?}", diags);
}

// `class Admin::AccountStatusesFilter < AccountStatusesFilter` — `<` is class
// inheritance, not a comparison; the namespace-qualified LHS must not be flagged.
#[test]
fn no_false_positive_for_class_inheritance() {
    let src = "class Admin::AccountStatusesFilter < AccountStatusesFilter\n  def foo\n    1\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessComparison.check_source(&ctx);
    assert!(diags.is_empty(), "class inheritance `<` falsely flagged: {:?}", diags);
}

// `computed_permissions & permissions != permissions` — the LHS of `!=` is the
// result of `computed_permissions & permissions` (bitwise AND), not just `permissions`.
// Must NOT be flagged as comparing `permissions` to itself.
#[test]
fn no_false_positive_for_bitwise_and_before_comparison() {
    let src = "errors.add(:k, :v) if acct.computed_permissions & permissions != permissions\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessComparison.check_source(&ctx);
    assert!(diags.is_empty(), "bitwise-AND compound LHS falsely flagged: {:?}", diags);
}

use rubric_core::{LintContext, Rule};
use rubric_rules::style::identical_conditional_branches::IdenticalConditionalBranches;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/identical_conditional_branches/offending.rb");
const PASSING: &str =
    include_str!("fixtures/style/identical_conditional_branches/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = IdenticalConditionalBranches.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/IdenticalConditionalBranches"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = IdenticalConditionalBranches.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on passing.rb; got: {:?}", diags);
}

#[test]
fn detects_identical_branches_with_complex_body() {
    let src = "if x > 0\n  render :ok\nelse\n  render :ok\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IdenticalConditionalBranches.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for identical render calls");
}

#[test]
fn no_violation_for_different_branches() {
    let src = "if x > 0\n  foo\nelse\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IdenticalConditionalBranches.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation; got: {:?}", diags);
}

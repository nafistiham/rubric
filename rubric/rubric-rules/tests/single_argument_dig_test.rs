use rubric_core::{LintContext, Rule};
use rubric_rules::style::single_argument_dig::SingleArgumentDig;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/single_argument_dig/offending.rb");
const PASSING: &str = include_str!("fixtures/style/single_argument_dig/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SingleArgumentDig.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/SingleArgumentDig"));
}

#[test]
fn no_violation_on_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = SingleArgumentDig.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}

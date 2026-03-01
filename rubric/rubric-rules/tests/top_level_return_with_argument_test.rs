use rubric_core::{LintContext, Rule};
use rubric_rules::lint::top_level_return_with_argument::TopLevelReturnWithArgument;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/top_level_return_with_argument/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TopLevelReturnWithArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/TopLevelReturnWithArgument"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  return 1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TopLevelReturnWithArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

use rubric_core::{LintContext, Rule};
use rubric_rules::style::single_line_methods::SingleLineMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/single_line_methods/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SingleLineMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/SingleLineMethods"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SingleLineMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code (empty method single line is exempt)");
}

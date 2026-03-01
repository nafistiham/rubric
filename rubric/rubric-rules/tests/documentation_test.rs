use rubric_core::{LintContext, Rule};
use rubric_rules::style::documentation::Documentation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/documentation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Documentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/Documentation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "# A well-documented class\nclass Foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Documentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

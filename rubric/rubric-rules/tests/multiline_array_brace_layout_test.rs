use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_array_brace_layout::MultilineArrayBraceLayout;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/multiline_array_brace_layout/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineArrayBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineArrayBraceLayout"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = [\n  1,\n  2,\n  3\n]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineArrayBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_hash_brace_layout::MultilineHashBraceLayout;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/multiline_hash_brace_layout/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineHashBraceLayout"));
}

#[test]
fn no_violation_on_clean() {
    let src = "h = {\n  a: 1,\n  b: 2\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

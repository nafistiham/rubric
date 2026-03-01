use rubric_core::{LintContext, Rule};
use rubric_rules::layout::def_end_alignment::DefEndAlignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/def_end_alignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/DefEndAlignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def bar\n  'bar'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

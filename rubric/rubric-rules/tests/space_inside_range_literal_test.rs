use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_inside_range_literal::SpaceInsideRangeLiteral;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_inside_range_literal/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceInsideRangeLiteral.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceInsideRangeLiteral"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = 1..10\ny = 1...20\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideRangeLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

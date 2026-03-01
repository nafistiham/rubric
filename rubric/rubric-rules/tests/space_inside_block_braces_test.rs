use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_inside_block_braces::SpaceInsideBlockBraces;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_inside_block_braces/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceInsideBlockBraces.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceInsideBlockBraces"));
}

#[test]
fn no_violation_on_clean() {
    let src = "[1, 2].each { |x| puts x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideBlockBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

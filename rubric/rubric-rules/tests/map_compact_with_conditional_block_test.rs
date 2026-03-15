use rubric_core::{LintContext, Rule};
use rubric_rules::style::map_compact_with_conditional_block::MapCompactWithConditionalBlock;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/map_compact_with_conditional_block/offending.rb");
const PASSING: &str =
    include_str!("fixtures/style/map_compact_with_conditional_block/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MapCompactWithConditionalBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(
        diags
            .iter()
            .all(|d| d.rule == "Style/MapCompactWithConditionalBlock"),
        "all diagnostics should have correct rule name"
    );
}

#[test]
fn no_violation_on_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = MapCompactWithConditionalBlock.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations in passing.rb, got: {:?}",
        diags
    );
}

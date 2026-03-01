use rubric_core::{LintContext, Rule};
use rubric_rules::style::unless_else::UnlessElse;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/unless_else/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/unless_else/corrected.rb");

#[test]
fn detects_unless_with_else() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnlessElse.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for `unless...else`, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/UnlessElse"));
}

#[test]
fn no_violation_with_if_else() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UnlessElse.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

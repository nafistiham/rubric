use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_self::RedundantSelf;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/redundant_self/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/redundant_self/corrected.rb");

#[test]
fn detects_redundant_self() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantSelf.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantSelf"));
}

#[test]
fn no_violation_without_redundant_self() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RedundantSelf.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

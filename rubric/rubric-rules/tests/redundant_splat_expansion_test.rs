use rubric_core::{LintContext, Rule};
use rubric_rules::lint::redundant_splat_expansion::RedundantSplatExpansion;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/redundant_splat_expansion/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantSplatExpansion.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/RedundantSplatExpansion"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo(bar)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSplatExpansion.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

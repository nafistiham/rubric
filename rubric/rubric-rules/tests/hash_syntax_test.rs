use rubric_core::{LintContext, Rule};
use rubric_rules::style::hash_syntax::HashSyntax;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/hash_syntax/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/hash_syntax/corrected.rb");

#[test]
fn detects_symbol_rocket_syntax() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = HashSyntax.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for :symbol => rocket syntax, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/HashSyntax"));
}

#[test]
fn no_violation_with_new_hash_syntax() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = HashSyntax.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

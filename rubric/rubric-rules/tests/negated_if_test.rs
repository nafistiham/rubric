use rubric_core::{LintContext, Rule};
use rubric_rules::style::negated_if::NegatedIf;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/negated_if/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/negated_if/corrected.rb");

#[test]
fn detects_negated_if() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for `if !`, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/NegatedIf"));
}

#[test]
fn no_violation_with_unless() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = NegatedIf.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── Modifier-form `if !` must be detected ────────────────────────────────────
// `do_something if !condition` — `if` is in modifier position after an
// expression. The current check only fires when `if` is the first token.
#[test]
fn detects_modifier_negated_if() {
    let src = "def foo\n  do_something if !condition\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "modifier `if !` should be flagged");
    assert!(diags.iter().all(|d| d.rule == "Style/NegatedIf"));
}

#[test]
fn detects_return_modifier_negated_if() {
    let src = "def bar\n  return x if !flag\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "modifier `if !` after return should be flagged");
}

// ── Block-form `if !` still fires ────────────────────────────────────────────
#[test]
fn still_detects_block_negated_if() {
    let src = "if !valid?\n  puts 'bad'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "block-form `if !` should still be flagged");
}

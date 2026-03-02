use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_around_operators::SpaceAroundOperators;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_around_operators/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_around_operators/corrected.rb");

#[test]
fn detects_missing_space_around_operators() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAroundOperators"));
}

#[test]
fn no_violation_with_spaces_around_operators() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── False positive: ||= compound assignment ──────────────────────────────────
#[test]
fn no_false_positive_for_or_assign() {
    let src = "x ||= default\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "||= falsely flagged: {:?}", diags);
}

// ── False positive: &&= compound assignment ──────────────────────────────────
#[test]
fn no_false_positive_for_and_assign() {
    let src = "x &&= condition\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "&&= falsely flagged: {:?}", diags);
}

// ── False positive: ** double-splat in method parameter ──────────────────────
#[test]
fn no_false_positive_for_double_splat_param() {
    let src = "def foo(**opts)\n  opts\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "**opts in params falsely flagged: {:?}", diags);
}

// ── False positive: operators inside a regex literal ─────────────────────────
#[test]
fn no_false_positive_for_regex_operators() {
    let src = "pattern = /a+b*/\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "regex operators falsely flagged: {:?}", diags);
}

// ── True positive: ||= without spaces should still be detected ───────────────
#[test]
fn detects_missing_space_around_or_assign() {
    let src = "x||=default\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(!diags.is_empty(), "x||=default should be flagged");
}

// ── True positive: ** exponentiation without spaces should still be detected ─
#[test]
fn detects_missing_space_around_exponentiation() {
    let src = "result = a**b\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(!diags.is_empty(), "a**b should be flagged");
}

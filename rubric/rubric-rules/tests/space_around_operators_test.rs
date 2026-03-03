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

// ── No violation: ** exponentiation uses no_space style (RuboCop default) ────
#[test]
fn no_violation_for_exponentiation_operator() {
    let src = "x = 10**6\ny = a**b\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for exponentiation, got: {:?}", diags);
}

// ── False positive: =~ regex match operator ───────────────────────────────────
#[test]
fn no_violation_for_regex_match_operator() {
    let src = "result = net =~ addr\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for =~ operator, got: {:?}", diags);
}

// ── False positive: setter method definition ──────────────────────────────────
#[test]
fn no_violation_for_setter_method_definition() {
    let src = "def bot=(val)\n  @bot = val\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for setter method def, got: {:?}", diags);
}

// ── False positive: setter method call (self.foo=val) ────────────────────────
#[test]
fn no_violation_for_setter_method_call() {
    let src = "self.discard_column=:deleted_at\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for setter method call, got: {:?}", diags);
}

// ── False positive: symbol literals like `:+`, `:-` (e.g., `reduce(:+)`) ────
#[test]
fn no_violation_for_symbol_operator_literal() {
    let src = "result = arr.reduce(:+)\nresult2 = arr.reduce(:-)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "symbol operator literals should not be flagged: {:?}", diags);
}

// ── False positive: %r{...} percent-regex content ────────────────────────────
#[test]
fn no_violation_for_percent_r_regex_content() {
    let src = "MENTION_RE = %r{(?<![=/[:word:]])@(([a-z0-9]+)(?:@[[:word:]]+)?)}i\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations inside %r{{}} regex, got: {:?}", diags);
}

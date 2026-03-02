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

// ── False positive: { after : in keyword argument hash ───────────────────────
// `key: {a: 1}` is a hash value, not a block brace. Must not fire.
#[test]
fn no_false_positive_for_hash_after_colon() {
    let src = "foo(key: {a: 1})\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideBlockBraces.check_source(&ctx);
    assert!(diags.is_empty(), "hash after colon falsely flagged: {:?}", diags);
}

// ── False positive: } closing a hash literal ─────────────────────────────────
// In `h = {key: 1}` the `}` closes a hash, not a block. Must not fire.
#[test]
fn no_false_positive_for_hash_closing_brace() {
    let src = "h = {key: 1}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideBlockBraces.check_source(&ctx);
    assert!(diags.is_empty(), "hash closing brace falsely flagged: {:?}", diags);
}

// ── True positive: block { without space should still fire ───────────────────
#[test]
fn still_detects_missing_space_after_block_brace() {
    let src = "[1, 2].each {|x| puts x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideBlockBraces.check_source(&ctx);
    assert!(!diags.is_empty(), "missing space after block `{{` should be flagged");
}

// ── True positive: block } without space should still fire ───────────────────
#[test]
fn still_detects_missing_space_before_block_closing_brace() {
    let src = "[1, 2].each { |x| puts x}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideBlockBraces.check_source(&ctx);
    assert!(!diags.is_empty(), "missing space before block `}}` should be flagged");
}

#[test]
fn no_violation_for_regex_quantifiers() {
    let src = "x = str.gsub(/(\\.d{2})(\\.d{7})/, '\\\\1-\\\\2')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideBlockBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in regex quantifiers, got: {:?}", diags);
}

#[test]
fn no_violation_for_percent_r_regex() {
    let src = "MENTION_RE = %r{(?<![=/[:word:]])@(([a-z0-9]+)(?:@[[:word:]]+)?)}i\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideBlockBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in %r{{}} regex, got: {:?}", diags);
}

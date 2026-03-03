use rubric_core::{LintContext, Rule};
use rubric_rules::lint::empty_block::EmptyBlock;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/empty_block/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/empty_block/corrected.rb");

#[test]
fn detects_empty_block() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/EmptyBlock"));
}

#[test]
fn no_violation_for_non_empty_block() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── False positive: empty hash literal `= {}` ─────────────────────────────────
#[test]
fn no_false_positive_for_empty_hash_literal_assignment() {
    let src = "opts = {}\nstat_hash = {}\n@weights = {}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(diags.is_empty(), "empty hash literal = {{}} falsely flagged: {:?}", diags);
}

// ── False positive: `{}` as default method argument ───────────────────────────
#[test]
fn no_false_positive_for_hash_default_param() {
    let src = "def kill(message, opts = {})\n  opts\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(diags.is_empty(), "hash default param falsely flagged: {:?}", diags);
}

// ── False positive: `{}` as rescue return value (standalone line) ──────────────
#[test]
fn no_false_positive_for_hash_as_rescue_value() {
    let src = "begin\n  foo\nrescue\n  {}\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(diags.is_empty(), "hash rescue return value falsely flagged: {:?}", diags);
}

// ── False positive: `|| {}` as fallback hash value ───────────────────────────
#[test]
fn no_false_positive_for_hash_or_default() {
    let src = "x = thing || {}\ny = a.b || {}\nz = opts.merge(opts.delete(:env) || {})\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(diags.is_empty(), "|| {{}} falsely flagged: {:?}", diags);
}

// ── False positive: empty lambda `->(val) {}` should not be flagged ──────────
#[test]
fn no_false_positive_for_empty_lambda() {
    let src = "VALIDATORS = {\n  Integer => ->(val) {},\n  Float => ->(val) {},\n  String => ->(val) {},\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(diags.is_empty(), "empty lambda ->(val) {{}} falsely flagged: {:?}", diags);
}

// ── True positive: empty block after method call should still be detected ──────
#[test]
fn still_detects_empty_block_after_method_call() {
    let src = "foo.bar {}\n[1,2,3].each {}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "empty block after method call not detected");
}

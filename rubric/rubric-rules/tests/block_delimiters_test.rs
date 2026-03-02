use rubric_core::{LintContext, Rule};
use rubric_rules::style::block_delimiters::BlockDelimiters;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/block_delimiters/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/block_delimiters/corrected.rb");

#[test]
fn detects_multiline_brace_block() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/BlockDelimiters"));
}

#[test]
fn no_violation_for_do_end_block() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── Multiline hash literal assigned with `=` must NOT fire ────────────────
// `CONSTANT = {\n  key: value,\n}.freeze` is a hash literal, not a block.
#[test]
fn no_false_positive_for_multiline_hash_literal_assignment() {
    let src = "LEGACY_MAP = {\n  'Foo' => :foo,\n  'Bar' => :bar,\n}.freeze\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "multiline hash literal assignment should not be flagged: {:?}", diags);
}

// ── Hash value inside outer hash must NOT fire ─────────────────────────────
// `mention: {\n  filterable: true,\n}.freeze` — inner hash in hash literal.
#[test]
fn no_false_positive_for_nested_hash_value() {
    let src = "PROPS = {\n  mention: {\n    filterable: true,\n  }.freeze,\n}.freeze\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "nested hash value should not be flagged: {:?}", diags);
}

// ── True positive: multiline brace block after method call still fires ─────
#[test]
fn still_detects_multiline_brace_block() {
    let src = "foo.each {\n  |x|\n  x + 1\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(!diags.is_empty(), "multiline brace block should still be flagged");
}

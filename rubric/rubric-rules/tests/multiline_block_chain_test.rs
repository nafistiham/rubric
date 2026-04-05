use rubric_core::{LintContext, Rule};
use rubric_rules::style::multiline_block_chain::MultilineBlockChain;
use std::path::Path;

fn check(src: &str) -> Vec<rubric_core::Diagnostic> {
    let ctx = LintContext::new(Path::new("test.rb"), src);
    MultilineBlockChain.check_source(&ctx)
}

#[test]
fn flags_brace_block_chain_flatten() {
    // Chaining a second block onto a multiline brace block — violation
    let src = "[1, 2].map { |x|\n  x * 2\n}.select { |x|\n  x > 1\n}\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "expected 1 violation for }}.select {{ }}, got: {:?}", diags);
    assert_eq!(diags[0].rule, "Style/MultilineBlockChain");
}

#[test]
fn message_is_correct() {
    let src = "[1, 2].map { |x|\n  x * 2\n}.select { |x|\n  x > 1\n}\n";
    let diags = check(src);
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("multi-line chains"));
}

#[test]
fn flags_do_end_block_chain() {
    // Chaining a second block onto a multiline do..end block — violation
    let src = "[1, 2].map do |x|\n  x * 2\nend.select do |x|\n  x > 1\nend\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "expected 1 violation for end.select do, got: {:?}", diags);
}

#[test]
fn flags_brace_block_chain_multiple_methods() {
    // Chaining a second block after intermediate method call — violation
    let src = "arr.map { |x|\n  x\n}.select { |x|\n  x > 0\n}\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "expected 1 violation for }}.select {{ }}, got: {:?}", diags);
}

#[test]
fn no_violation_for_clean_brace_close() {
    let src = "arr.map { |x|\n  x\n}\n";
    let diags = check(src);
    assert!(diags.is_empty(), "}} alone should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_clean_end() {
    let src = "[1].map do |x|\n  x\nend\n";
    let diags = check(src);
    assert!(diags.is_empty(), "end alone should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_freeze_chain() {
    // .freeze is idiomatic and accepted
    let src = "[1, 2].map { |x|\n  x\n}.freeze\n";
    let diags = check(src);
    assert!(diags.is_empty(), "}}.freeze should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_to_a_chain() {
    let src = "[1, 2].map { |x|\n  x\n}.to_a\n";
    let diags = check(src);
    assert!(diags.is_empty(), "}}.to_a should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_to_h_chain() {
    let src = "hash.map { |k, v|\n  [k, v]\n}.to_h\n";
    let diags = check(src);
    assert!(diags.is_empty(), "}}.to_h should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_to_s_chain() {
    let src = "arr.map { |x|\n  x\n}.to_s\n";
    let diags = check(src);
    assert!(diags.is_empty(), "}}.to_s should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_inspect_chain() {
    let src = "arr.map { |x|\n  x\n}.inspect\n";
    let diags = check(src);
    assert!(diags.is_empty(), "}}.inspect should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_indented_block_chain() {
    // Indented multiline block chained with a second block — violation
    let src = "  arr.map { |x|\n    x\n  }.select { |x|\n    x > 0\n  }\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "indented }}.select {{ }} should be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_comment_line() {
    let src = "# }.flatten\n";
    let diags = check(src);
    assert!(diags.is_empty(), "}}.flatten in comment should not be flagged, got: {:?}", diags);
}

#[test]
fn rule_name_is_correct() {
    assert_eq!(MultilineBlockChain.name(), "Style/MultilineBlockChain");
}

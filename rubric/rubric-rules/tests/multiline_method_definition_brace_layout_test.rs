use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_method_definition_brace_layout::MultilineMethodDefinitionBraceLayout;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/multiline_method_definition_brace_layout/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineMethodDefinitionBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineMethodDefinitionBraceLayout"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo(\n  bar,\n  baz\n)\n  bar + baz\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodDefinitionBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// ── Single-line def with body ending in `)` must NOT fire ─────────────────
// `def city(options: {})` is a complete single-line def.
// The method body line `parse(...)` ends with `)` but is not part of the signature.
#[test]
fn no_false_positive_for_single_line_def_body_ending_in_paren() {
    let src = "def city(options: {})\n  parse(options[:x] ? 'a' : 'b')\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodDefinitionBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "single-line def body ending in ) should not be flagged: {:?}", diags);
}

// ── Method call ending in `)` deep in method body must NOT fire ────────────
#[test]
fn no_false_positive_for_method_call_in_body() {
    let src = "def street_address(include_secondary: false)\n  numerify(parse('address') + (include_secondary ? ' #{secondary}' : ''))\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodDefinitionBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "method call in body should not be flagged: {:?}", diags);
}

// ── True positive: actual multiline def still fires ───────────────────────
#[test]
fn still_detects_multiline_def_closing_paren() {
    let src = "def foo(bar,\n        baz)\n  bar + baz\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodDefinitionBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "multiline def closing paren should still be flagged");
}

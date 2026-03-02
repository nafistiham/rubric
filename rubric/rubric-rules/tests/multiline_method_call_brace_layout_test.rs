use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_method_call_brace_layout::MultilineMethodCallBraceLayout;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/multiline_method_call_brace_layout/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineMethodCallBraceLayout"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo(\n  bar,\n  baz\n)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// ── False positive: symmetrical style — opening ( has first arg on same line ─
// `foo(arg1,\n    arg2)` is valid `symmetrical` style. Must not fire.
#[test]
fn no_false_positive_for_symmetrical_style() {
    let src = "foo(arg1,\n    arg2)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "symmetrical-style call falsely flagged: {:?}", diags);
}

// ── False positive: method call with parens in string literal on prior line ──
#[test]
fn no_false_positive_for_paren_in_string() {
    let src = "puts \"some (text)\"\nresult = foo(x)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "paren in string on prior line falsely flagged: {:?}", diags);
}

// ── True positive: bare opening `(` means ) must be on own line ──────────────
#[test]
fn detects_bare_open_paren_style_violation() {
    let src = "foo(\n  arg1,\n  arg2)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "bare-open-paren style with ) on arg line should be flagged");
}

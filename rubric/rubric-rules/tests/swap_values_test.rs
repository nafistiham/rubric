use rubric_core::{LintContext, Rule};
use rubric_rules::style::swap_values::SwapValues;
use std::path::Path;

fn check(src: &str) -> Vec<rubric_core::Diagnostic> {
    let ctx = LintContext::new(Path::new("test.rb"), src);
    SwapValues.check_source(&ctx)
}

#[test]
fn flags_classic_swap_pattern() {
    let src = "tmp = a\na = b\nb = tmp\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "expected 1 violation, got: {:?}", diags);
    assert_eq!(diags[0].rule, "Style/SwapValues");
}

#[test]
fn message_is_correct() {
    let src = "tmp = a\na = b\nb = tmp\n";
    let diags = check(src);
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("parallel assignment"));
}

#[test]
fn flags_first_line_of_swap() {
    // The diagnostic should be on line 1 (the tmp assignment)
    let src = "tmp = a\na = b\nb = tmp\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SwapValues.check_source(&ctx);
    assert_eq!(diags.len(), 1);
    // The range start should be at offset 0 (beginning of line 1)
    assert_eq!(diags[0].range.start, 0);
}

#[test]
fn flags_swap_with_underscore_vars() {
    let src = "_tmp = foo_bar\nfoo_bar = baz_qux\nbaz_qux = _tmp\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "expected 1 violation for underscore vars swap, got: {:?}", diags);
}

#[test]
fn no_violation_when_not_swap_pattern() {
    // Not a swap — tmp not used in line 3
    let src = "tmp = a\na = b\nb = c\n";
    let diags = check(src);
    assert!(diags.is_empty(), "not a swap pattern, should not flag, got: {:?}", diags);
}

#[test]
fn no_violation_when_lines_not_consecutive() {
    let src = "tmp = a\n\na = b\nb = tmp\n";
    let diags = check(src);
    assert!(diags.is_empty(), "non-consecutive lines should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_when_tmp_is_reused() {
    // Line 3 reassigns tmp instead of original var
    let src = "tmp = a\na = b\ntmp = b\n";
    let diags = check(src);
    assert!(diags.is_empty(), "not a swap pattern, should not flag, got: {:?}", diags);
}

#[test]
fn no_violation_for_comment_lines() {
    let src = "# tmp = a\na = b\nb = tmp\n";
    let diags = check(src);
    assert!(diags.is_empty(), "comment line should not trigger swap detection, got: {:?}", diags);
}

#[test]
fn no_violation_when_rhs_is_method_call() {
    // RHS is not a simple var
    let src = "tmp = a.upcase\na = b\nb = tmp\n";
    let diags = check(src);
    assert!(diags.is_empty(), "method call on RHS should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_indented_swap() {
    let src = "  tmp = a\n  a = b\n  b = tmp\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "indented swap should still be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_when_tmp_in_string() {
    let src = "x = \"tmp = a\"\na = b\nb = x\n";
    let diags = check(src);
    assert!(diags.is_empty(), "string content should not trigger swap, got: {:?}", diags);
}

#[test]
fn flags_instance_var_swap() {
    let src = "tmp = @foo\n@foo = @bar\n@bar = tmp\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "expected 1 violation for instance var swap, got: {:?}", diags);
}

#[test]
fn rule_name_is_correct() {
    assert_eq!(SwapValues.name(), "Style/SwapValues");
}

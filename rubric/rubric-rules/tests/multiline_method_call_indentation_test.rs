use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_method_call_indentation::MultilineMethodCallIndentation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/multiline_method_call_indentation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/multiline_method_call_indentation/corrected.rb");

#[test]
fn detects_trailing_dot_in_chained_call() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for trailing dots, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineMethodCallIndentation"));
}

#[test]
fn no_violation_with_leading_dots() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── Doc comments ending with period must NOT fire ──────────────────────────
// YARD-style `# description.` lines legitimately end with `.` (sentence end).
// The trailing-dot check must skip all comment lines.
#[test]
fn no_false_positive_for_doc_comment_ending_with_period() {
    let src = "##\n# Produces the name of a city.\n#\n# @return [String]\ndef city\n  fetch('city')\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "doc comment ending with period should not be flagged: {:?}", diags);
}

// ── Inline comments ending with period must NOT fire ───────────────────────
// `x = foo # Gets the value.` — the `.` is inside the comment, not in code.
#[test]
fn no_false_positive_for_inline_comment_ending_with_period() {
    let src = "x = foo # Gets the value.\ny = bar\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "inline comment ending with period should not be flagged: {:?}", diags);
}

// ── True positive: code line with trailing dot still fires ─────────────────
#[test]
fn still_detects_trailing_dot_code_line() {
    let src = "result = foo.\n  bar\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "trailing dot in code should still be flagged");
}

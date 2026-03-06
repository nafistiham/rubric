use rubric_core::{LintContext, Rule};
use rubric_rules::lint::ambiguous_operator::AmbiguousOperator;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/ambiguous_operator/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/ambiguous_operator/corrected.rb");

#[test]
fn detects_ambiguous_operator() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AmbiguousOperator.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/AmbiguousOperator"));
}

#[test]
fn no_violation_for_unambiguous_operator() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = AmbiguousOperator.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── False positive: `&` inside string literals ───────────────────────────────
// HTML entities like `&amp;` inside 'string' or "string" must not be flagged.
#[test]
fn no_false_positive_for_ampersand_in_string_literal() {
    let src = "text = 'Lorem &amp; ipsum &#x2764;'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = AmbiguousOperator.check_source(&ctx);
    assert!(diags.is_empty(), "& inside string must not be flagged: {:?}", diags);
}

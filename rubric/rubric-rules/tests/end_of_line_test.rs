use rubric_core::{LintContext, Rule};
use rubric_rules::layout::end_of_line::EndOfLine;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/end_of_line/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/end_of_line/corrected.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EndOfLine.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/EndOfLine"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = 1\ny = 2\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndOfLine.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_violation_on_corrected() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EndOfLine.check_source(&ctx);
    assert!(diags.is_empty(), "corrected.rb should have no violations");
}

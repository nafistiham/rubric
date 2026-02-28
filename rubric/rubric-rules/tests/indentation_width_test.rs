use rubric_core::{LintContext, Rule};
use rubric_rules::layout::indentation_width::IndentationWidth;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/indentation_width/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/indentation_width/corrected.rb");

#[test]
fn detects_wrong_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Layout/IndentationWidth"));
}

#[test]
fn no_violation_on_correct_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty());
}

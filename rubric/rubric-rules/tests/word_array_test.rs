use rubric_core::{LintContext, Rule};
use rubric_rules::style::word_array::WordArray;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/word_array/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/word_array/corrected.rb");

#[test]
fn detects_string_array_that_should_use_percent_w() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = WordArray.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for string array, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/WordArray"));
}

#[test]
fn no_violation_with_percent_w_syntax() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = WordArray.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

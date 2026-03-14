use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_interpolation::RedundantInterpolation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/redundant_interpolation/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/redundant_interpolation/clean.rb");

#[test]
fn detects_pure_interpolation_strings() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantInterpolation"));
}

#[test]
fn no_violation_for_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = RedundantInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn does_not_flag_interpolation_with_prefix_text() {
    let src = "x = \"prefix #{var}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "prefix text should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_multiple_interpolations() {
    let src = "x = \"#{a} and #{b}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "multiple interpolations should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_pure_interpolation_with_correct_message() {
    let src = "foo = \"#{bar}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected a violation");
    assert!(
        diags[0].message.contains("to_s"),
        "message should mention to_s, got: {}",
        diags[0].message
    );
}

#[test]
fn does_not_flag_interpolation_in_comment() {
    let src = "# \"#{bar}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}

#[test]
fn counts_one_violation_per_pure_interpolation() {
    let src = "a = \"#{x}\"\nb = \"#{y}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantInterpolation.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations, got: {:?}", diags);
}

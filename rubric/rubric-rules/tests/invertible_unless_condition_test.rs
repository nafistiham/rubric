use rubric_core::{LintContext, Rule};
use rubric_rules::style::invertible_unless_condition::InvertibleUnlessCondition;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/invertible_unless_condition/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/invertible_unless_condition/corrected.rb");

#[test]
fn detects_unless_negation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = InvertibleUnlessCondition.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violations for `unless !`, got none"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/InvertibleUnlessCondition"));
}

#[test]
fn no_violation_for_positive_unless() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = InvertibleUnlessCondition.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for `if`, got: {:?}",
        diags
    );
}

#[test]
fn detects_block_form_unless_negation() {
    let src = "unless !valid?\n  process\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InvertibleUnlessCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "block-form `unless !` should be flagged");
    assert_eq!(diags[0].rule, "Style/InvertibleUnlessCondition");
}

#[test]
fn detects_modifier_form_unless_negation() {
    let src = "do_something unless !condition\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InvertibleUnlessCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "modifier-form `unless !` should be flagged");
}

#[test]
fn no_violation_for_unless_with_not_equals() {
    // `unless x != y` — `!` followed by `=` is `!=` operator, not negation
    let src = "unless x != y\n  do_it\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InvertibleUnlessCondition.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`unless x != y` should not be flagged (not is `!=` operator), got: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_plain_unless() {
    let src = "unless condition\n  do_it\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InvertibleUnlessCondition.check_source(&ctx);
    assert!(diags.is_empty(), "plain `unless` should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_comment_line() {
    let src = "# unless !foo\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InvertibleUnlessCondition.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_unless_double_negation_method() {
    // `unless !user.active?` — method call with `?`
    let src = "raise 'inactive' unless !user.active?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InvertibleUnlessCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "`unless !method?` should be flagged");
}

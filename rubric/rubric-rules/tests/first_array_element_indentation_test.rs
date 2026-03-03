use rubric_core::{LintContext, Rule};
use rubric_rules::layout::first_array_element_indentation::FirstArrayElementIndentation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/first_array_element_indentation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FirstArrayElementIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/FirstArrayElementIndentation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = [\n  1,\n  2,\n]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstArrayElementIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Elements aligned to bracket column+2 inside a method call: `sample([`
// This is valid `special_inside_parentheses` style alignment
#[test]
fn no_false_positive_for_inline_bracket_alignment() {
    // `sample([` has `[` at col 17; element at col 19 = bracket_col+2
    let src = concat!(
        "          sample([\n",
        "                   Char.prepare(Name.first_name),\n",
        "                 ])\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstArrayElementIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "inline bracket alignment falsely flagged: {:?}", diags);
}

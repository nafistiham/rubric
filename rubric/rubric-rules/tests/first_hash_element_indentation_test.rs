use rubric_core::{LintContext, Rule};
use rubric_rules::layout::first_hash_element_indentation::FirstHashElementIndentation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/first_hash_element_indentation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/first_hash_element_indentation/corrected.rb");

#[test]
fn detects_missing_indentation_in_hash() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FirstHashElementIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for unindented hash elements, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/FirstHashElementIndentation"));
}

#[test]
fn no_violation_with_correct_hash_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = FirstHashElementIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// Hash literals inside YARD/RDoc comment examples must not be flagged
#[test]
fn no_false_positive_for_hash_in_comment() {
    let src = concat!(
        "# @example\n",
        "#   Faker::Bird.name #=> {\n",
        "#     order: 'Accipitriformes',\n",
        "#     common_name: 'Hawk'\n",
        "#   }\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstHashElementIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "hash in comment falsely flagged: {:?}", diags);
}

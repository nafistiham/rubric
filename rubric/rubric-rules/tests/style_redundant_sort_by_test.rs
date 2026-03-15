use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/style/redundant_sort_by.rs"]
mod redundant_sort_by;
use redundant_sort_by::RedundantSortBy;

const OFFENDING: &str = include_str!("fixtures/style/redundant_sort_by/offending.rb");
const PASSING: &str = include_str!("fixtures/style/redundant_sort_by/passing.rb");

#[test]
fn detects_redundant_sort_by() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantSortBy.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violations for sort_by {{ |x| x }}, got none"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantSortBy"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = RedundantSortBy.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for passing code, got: {:?}",
        diags
    );
}

#[test]
fn flags_identity_sort_by_block() {
    let src = "arr.sort_by { |x| x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSortBy.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for sort_by {{ |x| x }}");
    assert!(
        diags[0].message.contains("sort"),
        "message should mention sort, got: {}",
        diags[0].message
    );
}

#[test]
fn does_not_flag_sort_by_with_method_call() {
    let src = "arr.sort_by { |x| x.name }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSortBy.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "sort_by {{ |x| x.name }} should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn does_not_flag_bare_sort() {
    let src = "arr.sort\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSortBy.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "bare sort should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn does_not_flag_sort_by_with_negation() {
    let src = "arr.sort_by { |x| -x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSortBy.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "sort_by with negation should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# arr.sort_by { |x| x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSortBy.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "comment should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn flags_item_identity_sort_by() {
    let src = "items.sort_by { |item| item }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSortBy.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violation for sort_by {{ |item| item }}"
    );
}

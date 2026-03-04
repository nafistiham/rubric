use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_after_comma::SpaceAfterComma;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_after_comma/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/space_after_comma/corrected.rb");

#[test]
fn detects_missing_space_after_comma() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAfterComma"));
}

#[test]
fn no_violation_with_spaces_after_comma() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(diags.is_empty());
}

#[test]
fn no_false_positive_on_comma_in_string() {
    let source = "foo(\"a,b\", \"c,d\")\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(diags.is_empty(), "should not flag commas inside strings, got: {:?}", diags);
}

// -- False positive: commas in inline comments ---------------------------------
#[test]
fn no_false_positive_for_comma_in_inline_comment() {
    let src = "x = foo # perform_async(1,2,3)\n# a comment with [Time,Range]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(diags.is_empty(), "commas in comments falsely flagged: {:?}", diags);
}

// -- False positive: commas inside backtick shell strings ---------------------
#[test]
fn no_false_positive_for_comma_in_backtick_string() {
    let src = "result = `ps -o pid,rss -p #{pid}`.strip\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(diags.is_empty(), "commas in backtick string falsely flagged: {:?}", diags);
}

// -- False positive: comma inside nested string in interpolation --------------
#[test]
fn no_false_positive_for_comma_in_nested_interpolated_string() {
    let src = "x = \"#{arr.join(\",\")}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(diags.is_empty(), "commas in interpolated string falsely flagged: {:?}", diags);
}

// -- False positive: commas inside regex quantifiers /foo{n,m}/ ---------------
#[test]
fn no_false_positive_for_regex_quantifier() {
    let src = "x = /foo{1,3}/\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(diags.is_empty(), "comma in regex quantifier falsely flagged: {:?}", diags);
}

// -- False positive: regex quantifier in assert_match call --------------------
// The method-call comma after the regex has a space so it must NOT be flagged.
// The comma inside {2,5} is inside a regex and must NOT be flagged either.
#[test]
fn no_false_positive_for_regex_in_assert() {
    let src = "assert_match(/^[a-z]{2,5}/, val)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "comma in regex quantifier inside assert_match falsely flagged: {:?}",
        diags
    );
}

// -- Regression: real missing-space violations still detected -----------------
#[test]
fn still_detects_missing_space_after_comma_in_args() {
    let src = "foo(a,b)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterComma.check_source(&ctx);
    assert_eq!(diags.len(), 1, "should detect missing space in args, got: {:?}", diags);
    assert_eq!(diags[0].rule, "Layout/SpaceAfterComma");
}

use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_around_keyword::SpaceAroundKeyword;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_around_keyword/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_around_keyword/corrected.rb");

#[test]
fn detects_keyword_without_space() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAroundKeyword.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAroundKeyword"));
}

#[test]
fn no_violation_with_space_around_keyword() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAroundKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_method_call_with_keyword_name() {
    // `.not(`, `.or(`, `.in(`, `.and(` are method calls, not keyword usage
    let src = "scope :foo, -> { where(status.not(nil)) }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for .not( method call, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_active_record_dot_not() {
    let src = "scope :active, -> { where.not(id: nil) }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for where.not( method call, got: {:?}", diags);
}

#[test]
fn still_detects_keyword_without_space_before_paren() {
    // `not(x)` at the start (not preceded by `.`) should still fire
    let src = "def foo\n  result = not(x)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundKeyword.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for not( without preceding dot, got none");
}

// Keywords inside string literals must not be flagged
// e.g. XPath expression `not(` inside a single-quoted string
#[test]
fn no_false_positive_for_keyword_inside_string() {
    let src = "tree.xpath('./text()|.//text()[not(ancestor)]').to_a\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "keyword inside string falsely flagged: {:?}", diags);
}

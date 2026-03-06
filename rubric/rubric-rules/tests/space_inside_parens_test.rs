use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_inside_parens::SpaceInsideParens;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_inside_parens/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_inside_parens/corrected.rb");

#[test]
fn detects_space_inside_parens() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceInsideParens.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceInsideParens"));
}

#[test]
fn no_violation_without_space_inside_parens() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceInsideParens.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_multiline_paren_close() {
    let src = "result = foo(\n  x,\n  y\n  )\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideParens.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for multiline paren close, got: {:?}", diags);
}

// FP: parens inside regex literals must not be flagged
#[test]
fn no_false_positive_for_paren_inside_regex() {
    let src = "line =~ /(?:Master|      ) PID:/\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideParens.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for paren in regex, got: {:?}", diags);
}

// FP: non-capturing regex group `(?:...)` after `=~`
#[test]
fn no_false_positive_for_non_capturing_group_in_regex() {
    let src = "m = str.match(/(?:foo|bar) baz/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideParens.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for non-capturing group in regex, got: {:?}", diags);
}

// FP: parens inside %r{...} multiline percent regex must not be flagged
#[test]
fn no_false_positive_for_paren_in_percent_regex() {
    let src = concat!(
        "PATTERN = %r{\n",
        "  (https?://)?     # optional protocol\n",
        "  ([a-z]+\\.)+     # domain parts\n",
        "}x\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideParens.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations inside %r{{...}} regex, got: {:?}", diags);
}

// FP: parens inside heredoc bodies must not be flagged
#[test]
fn no_false_positive_for_paren_in_heredoc_body() {
    let src = concat!(
        "  create_view \"accts\", sql_definition: <<-SQL\n",
        "    SELECT CROSS JOIN LATERAL ( SELECT id FROM foo )\n",
        "  SQL\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideParens.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations inside heredoc SQL body, got: {:?}", diags);
}

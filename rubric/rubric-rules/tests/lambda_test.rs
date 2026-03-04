use rubric_core::{LintContext, Rule};
use rubric_rules::style::lambda::Lambda;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/lambda/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/lambda/corrected.rb");

#[test]
fn detects_lambda_keyword() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Lambda.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/Lambda"));
}

#[test]
fn no_violation_for_stabby_lambda() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = Lambda.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// FP: `lambda do ... end` — rubocop only flags single-line `lambda { }`, not do/end form
#[test]
fn no_false_positive_for_lambda_do_end() {
    let src = concat!(
        "TRANSFORMER = lambda do |env|\n",
        "  env[:node].remove_attribute('translate')\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Lambda.check_source(&ctx);
    assert!(diags.is_empty(), "`lambda do` falsely flagged: {:?}", diags);
}

// FP: multiline `lambda { |args| ... }` — rubocop only flags single-line form
#[test]
fn no_false_positive_for_multiline_lambda_brace() {
    let src = concat!(
        "scope :tagged_with_all, lambda { |tag_ids|\n",
        "  Array(tag_ids).reduce(self) { |r, id| r.where(tag_id: id) }\n",
        "}\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Lambda.check_source(&ctx);
    assert!(diags.is_empty(), "multiline `lambda {{` falsely flagged: {:?}", diags);
}

// True positive: single-line `lambda { }` must still be flagged
#[test]
fn detects_single_line_lambda_brace() {
    let src = "fn = lambda { |x| x + 1 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Lambda.check_source(&ctx);
    assert!(!diags.is_empty(), "single-line `lambda {{ }}` should be flagged");
}

use rubric_core::{LintContext, Rule};
use rubric_rules::style::multiline_if_then::MultilineIfThen;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/multiline_if_then/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/multiline_if_then/clean.rb");

#[test]
fn detects_then_in_multiline_if() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineIfThen.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations (if then + unless then)");
    assert!(diags.iter().all(|d| d.rule == "Style/MultilineIfThen"));
}

#[test]
fn no_violation_for_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = MultilineIfThen.check_source(&ctx);
    assert_eq!(diags.len(), 0, "expected no violations for clean code");
}

#[test]
fn no_violation_for_one_liner_with_end() {
    let source = "if cond then do_it end\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = MultilineIfThen.check_source(&ctx);
    assert_eq!(diags.len(), 0, "one-liner with end should not be flagged");
}

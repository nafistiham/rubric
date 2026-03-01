use rubric_core::{LintContext, Rule};
use rubric_rules::style::stderr_puts::StderrPuts;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/stderr_puts/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/stderr_puts/corrected.rb");

#[test]
fn detects_stderr_puts() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = StderrPuts.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/StderrPuts"));
}

#[test]
fn no_violation_for_warn() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = StderrPuts.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

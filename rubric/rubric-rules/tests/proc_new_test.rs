use rubric_core::{LintContext, Rule};
use rubric_rules::style::proc_new::ProcNew;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/proc_new/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/proc_new/corrected.rb");

#[test]
fn detects_proc_new() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ProcNew.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/Proc"));
}

#[test]
fn no_violation_for_proc_literal() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = ProcNew.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

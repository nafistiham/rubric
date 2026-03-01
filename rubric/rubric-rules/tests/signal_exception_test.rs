use rubric_core::{LintContext, Rule};
use rubric_rules::style::signal_exception::SignalException;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/signal_exception/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/signal_exception/corrected.rb");

#[test]
fn detects_fail_keyword() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SignalException.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/SignalException"));
}

#[test]
fn no_violation_for_raise_keyword() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SignalException.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

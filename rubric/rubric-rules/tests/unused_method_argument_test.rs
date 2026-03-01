use rubric_core::{LintContext, Rule};
use rubric_rules::lint::unused_method_argument::UnusedMethodArgument;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/unused_method_argument/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/unused_method_argument/corrected.rb");

#[test]
fn detects_unused_method_argument() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnusedMethodArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for unused arg, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnusedMethodArgument"));
}

#[test]
fn no_violation_with_all_args_used_or_prefixed() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UnusedMethodArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

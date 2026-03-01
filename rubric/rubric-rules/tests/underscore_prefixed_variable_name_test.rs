use rubric_core::{LintContext, Rule};
use rubric_rules::lint::underscore_prefixed_variable_name::UnderscorePrefixedVariableName;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/underscore_prefixed_variable_name/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnderscorePrefixedVariableName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnderscorePrefixedVariableName"));
}

#[test]
fn no_violation_on_clean() {
    let src = "_foo = 1\n# intentionally unused\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnderscorePrefixedVariableName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

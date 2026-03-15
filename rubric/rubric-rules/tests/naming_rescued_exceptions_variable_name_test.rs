use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/naming/rescued_exceptions_variable_name.rs"]
mod rescued_exceptions_variable_name;
use rescued_exceptions_variable_name::RescuedExceptionsVariableName;

const OFFENDING: &str = include_str!(
    "fixtures/naming/rescued_exceptions_variable_name/offending.rb"
);
const CLEAN: &str =
    include_str!("fixtures/naming/rescued_exceptions_variable_name/clean.rb");

#[test]
fn detects_non_standard_rescue_variable() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RescuedExceptionsVariableName.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violations for non-standard rescue variable names, got none"
    );
    assert!(diags
        .iter()
        .all(|d| d.rule == "Naming/RescuedExceptionsVariableName"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = RescuedExceptionsVariableName.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for standard rescue variable 'e', got: {:?}",
        diags
    );
}

#[test]
fn flags_exception_variable_name() {
    let src = "rescue StandardError => exception\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescuedExceptionsVariableName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for 'exception'");
    assert!(
        diags[0].message.contains("exception"),
        "message should mention the bad name, got: {}",
        diags[0].message
    );
}

#[test]
fn flags_bare_rescue_with_non_e_variable() {
    let src = "rescue => error\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescuedExceptionsVariableName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for bare rescue => error");
}

#[test]
fn allows_e_variable() {
    let src = "rescue StandardError => e\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescuedExceptionsVariableName.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "rescue => e should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn allows_multiple_exception_types_with_e() {
    let src = "rescue TypeError, ArgumentError => e\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescuedExceptionsVariableName.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "rescue TypeError, ArgumentError => e should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# rescue => exception\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescuedExceptionsVariableName.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "comment should not be flagged, got: {:?}",
        diags
    );
}

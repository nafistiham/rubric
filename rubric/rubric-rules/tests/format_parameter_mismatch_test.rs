use rubric_core::{LintContext, Rule};
use rubric_rules::lint::format_parameter_mismatch::FormatParameterMismatch;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/format_parameter_mismatch/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FormatParameterMismatch.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/FormatParameterMismatch"));
}

#[test]
fn no_violation_on_clean() {
    let src = "sprintf(\"%s %s\", name, age)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FormatParameterMismatch.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Width/flag specifiers like `%09d` or `%-10s` must be recognized
#[test]
fn no_false_positive_for_width_flag_specifiers() {
    let src = "format('%09d', rand(10**9)).gsub(/pattern/, 'repl')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FormatParameterMismatch.check_source(&ctx);
    assert!(diags.is_empty(), "width-flag specifier %09d falsely flagged: {:?}", diags);
}

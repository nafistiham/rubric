use rubric_core::{LintContext, Rule};
use rubric_rules::style::string_concatenation::StringConcatenation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/string_concatenation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = StringConcatenation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/StringConcatenation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "greeting = \"Hello, #{name}\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StringConcatenation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

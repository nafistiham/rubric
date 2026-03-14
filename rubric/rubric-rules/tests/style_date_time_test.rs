use rubric_core::{LintContext, Rule};
use rubric_rules::style::date_time::DateTime;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/date_time/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/date_time/clean.rb");

#[test]
fn detects_datetime_now() {
    let src = "dt = DateTime.now\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DateTime.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for DateTime.now");
    assert_eq!(diags[0].rule, "Style/DateTime");
    assert!(diags[0].message.contains("Time"));
}

#[test]
fn detects_datetime_new() {
    let src = "dt = DateTime.new(2023, 1, 1)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DateTime.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for DateTime.new");
}

#[test]
fn no_violation_on_time() {
    let src = "t = Time.now\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DateTime.check_source(&ctx);
    assert!(diags.is_empty(), "Time.now should not be flagged");
}

#[test]
fn no_violation_on_class_definition() {
    let src = "class DateTime\n  # custom\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DateTime.check_source(&ctx);
    assert!(diags.is_empty(), "class DateTime definition should not be flagged");
}

#[test]
fn no_violation_in_comment() {
    let src = "# DateTime.now is bad\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DateTime.check_source(&ctx);
    assert!(diags.is_empty(), "DateTime in comment should not be flagged");
}

#[test]
fn no_violation_in_string() {
    let src = r#"msg = "use DateTime.now here"
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DateTime.check_source(&ctx);
    assert!(diags.is_empty(), "DateTime in string should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_in_require() {
    let src = "require 'date_time_extra'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DateTime.check_source(&ctx);
    assert!(diags.is_empty(), "DateTime in require should not be flagged");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DateTime.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/DateTime"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = DateTime.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb, got: {:?}", diags);
}

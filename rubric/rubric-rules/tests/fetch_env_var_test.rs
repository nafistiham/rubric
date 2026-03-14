use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/style/fetch_env_var.rs"]
mod fetch_env_var;
use fetch_env_var::FetchEnvVar;

const OFFENDING: &str = include_str!("fixtures/style/fetch_env_var/offending.rb");
const PASSING: &str = include_str!("fixtures/style/fetch_env_var/passing.rb");

#[test]
fn detects_env_bracket_access() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/FetchEnvVar"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_single_quoted_key() {
    let src = "db_url = ENV['DATABASE_URL']\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(!diags.is_empty(), "ENV['KEY'] should be flagged");
    assert!(diags[0].message.contains("ENV.fetch"));
}

#[test]
fn flags_double_quoted_key() {
    let src = "secret = ENV[\"SECRET_KEY\"]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(!diags.is_empty(), "ENV[\"KEY\"] should be flagged");
}

#[test]
fn does_not_flag_env_fetch() {
    let src = "db_url = ENV.fetch('DATABASE_URL')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(diags.is_empty(), "ENV.fetch should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_env_fetch_with_default() {
    let src = "port = ENV.fetch('PORT', '3000')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(diags.is_empty(), "ENV.fetch with default should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment() {
    let src = "# ENV['KEY'] is bad\ndb = ENV.fetch('KEY')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_inside_string() {
    let src = r#"msg = "use ENV['KEY'] carefully"
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(diags.is_empty(), "string contents should not be flagged, got: {:?}", diags);
}

#[test]
fn skips_heredoc_body() {
    let src = "doc = <<~TEXT\n  ENV['KEY'] is used here\nTEXT\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FetchEnvVar.check_source(&ctx);
    assert!(diags.is_empty(), "heredoc body should not be flagged, got: {:?}", diags);
}

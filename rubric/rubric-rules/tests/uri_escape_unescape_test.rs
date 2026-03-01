use rubric_core::{LintContext, Rule};
use rubric_rules::lint::uri_escape_unescape::UriEscapeUnescape;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/uri_escape_unescape/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UriEscapeUnescape.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UriEscapeUnescape"));
}

#[test]
fn no_violation_on_clean() {
    let src = "URI::DEFAULT_PARSER.escape('hello world')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UriEscapeUnescape.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

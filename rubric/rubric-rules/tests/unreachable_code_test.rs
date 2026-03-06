use rubric_core::{LintContext, Rule};
use rubric_rules::lint::unreachable_code::UnreachableCode;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/unreachable_code/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/unreachable_code/corrected.rb");

#[test]
fn detects_unreachable_code() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnreachableCode.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnreachableCode"));
}

#[test]
fn no_violation_for_reachable_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UnreachableCode.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_for_guard_return_unless() {
    let src = "def refresh_webfinger!\n  return unless last_webfingered_at.present?\n  AccountRefreshWorker.perform_in(rand(6.hours.to_i), id)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnreachableCode.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for guard return unless, got: {:?}", diags);
}

#[test]
fn no_violation_for_guard_return_if() {
    let src = "def foo\n  return if condition?\n  do_something\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnreachableCode.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for guard return if, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_hash_next_key() {
    let src = "def foo\n  opts = {\n    next: next_page,\n    break: stop_point\n  }\n  opts\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnreachableCode.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for hash with next:/break: keys, got: {:?}", diags);
}

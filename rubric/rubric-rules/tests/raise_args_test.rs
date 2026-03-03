use rubric_core::{LintContext, Rule};
use rubric_rules::style::raise_args::RaiseArgs;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/raise_args/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/raise_args/corrected.rb");

#[test]
fn detects_raise_with_new_and_args() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for .new(msg) style, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/RaiseArgs"));
}

#[test]
fn no_violation_for_exploded_comma_style() {
    // `raise ExceptionClass, "msg"` is the preferred exploded style — must not flag
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for exploded style, got: {:?}", diags);
}

#[test]
fn no_violation_for_raise_new_without_args() {
    // `raise SomeError.new` with no message is NOT flagged by RuboCop exploded style
    let src = "raise SomeError.new\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for .new with no args, got: {:?}", diags);
}

#[test]
fn no_violation_for_bare_raise() {
    // `raise` alone (re-raise) must never be flagged
    let src = "raise\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for bare raise, got: {:?}", diags);
}

#[test]
fn no_violation_for_raise_variable() {
    // `raise e` or `raise err` — lowercase, not a class literal
    let src = "raise e\nraise err\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for raise with variable, got: {:?}", diags);
}

#[test]
fn no_violation_for_raise_plain_string() {
    // `raise "msg"` — no class involved
    let src = "raise \"something bad happened\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for raise with plain string, got: {:?}", diags);
}

#[test]
fn detects_raise_new_with_string_arg() {
    // `raise RuntimeError.new("msg")` should be flagged — use exploded comma style
    let src = "raise RuntimeError.new(\"something went wrong\")\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RaiseArgs.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected exactly 1 violation, got: {:?}", diags);
    assert_eq!(diags[0].rule, "Style/RaiseArgs");
}

#[test]
fn detects_raise_new_with_interpolated_arg() {
    // `raise ArgumentError.new("bad: #{val}")` should also be flagged
    let src = "raise ArgumentError.new(\"bad: #{val}\")\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RaiseArgs.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected exactly 1 violation, got: {:?}", diags);
}

#[test]
fn sidekiq_style_comma_is_not_flagged() {
    // Sidekiq uses: `raise ArgumentError, "requires a block" unless block_given?`
    // This is the RuboCop-preferred exploded style and must produce zero violations
    let src = concat!(
        "raise ArgumentError, \"requires a block\" unless block_given?\n",
        "raise NotImplementedError, \"must implement #call\"\n",
        "raise Sidekiq::Error, \"connection failed\"\n",
        "raise ::RuntimeError, \"global ns\"\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RaiseArgs.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "sidekiq-style comma raises must not be flagged, got: {:?}",
        diags
    );
}

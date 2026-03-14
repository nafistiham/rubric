use rubric_core::{LintContext, Rule};
use rubric_rules::style::global_vars::GlobalVars;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/global_vars/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/global_vars/clean.rb");

#[test]
fn detects_custom_global() {
    let src = "$my_global = 42\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = GlobalVars.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for $my_global");
    assert!(diags[0].message.contains("global variables"));
}

#[test]
fn no_violation_for_builtin_stdout() {
    let src = "puts $stdout\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = GlobalVars.check_source(&ctx);
    assert!(diags.is_empty(), "$stdout is a built-in, should not be flagged");
}

#[test]
fn no_violation_for_builtin_load_path() {
    let src = "$LOAD_PATH.push(dir)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = GlobalVars.check_source(&ctx);
    assert!(diags.is_empty(), "$LOAD_PATH is a built-in");
}

#[test]
fn no_violation_in_string() {
    let src = "msg = \"$my_var\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = GlobalVars.check_source(&ctx);
    assert!(diags.is_empty(), "global inside string should not be flagged");
}

#[test]
fn no_violation_for_perl_digit_globals() {
    let src = "puts $1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = GlobalVars.check_source(&ctx);
    assert!(diags.is_empty(), "$1 is a perl backref, handled separately");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = GlobalVars.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/GlobalVars"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = GlobalVars.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb");
}

use rubric_core::{LintContext, Rule};
use rubric_rules::style::perl_backrefs::PerlBackrefs;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/perl_backrefs/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/perl_backrefs/clean.rb");

#[test]
fn detects_perl_backrefs() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = PerlBackrefs.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations ($1 and $2)");
    assert!(diags.iter().all(|d| d.rule == "Style/PerlBackrefs"));
}

#[test]
fn no_violation_for_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = PerlBackrefs.check_source(&ctx);
    assert_eq!(diags.len(), 0, "expected no violations for clean code");
}

#[test]
fn skips_dollar_zero() {
    let source = "puts $0\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = PerlBackrefs.check_source(&ctx);
    assert_eq!(diags.len(), 0, "$0 (program name) should not be flagged");
}

#[test]
fn skips_inside_string() {
    let source = "x = \"$1 match\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = PerlBackrefs.check_source(&ctx);
    assert_eq!(diags.len(), 0, "$1 inside string should not be flagged");
}

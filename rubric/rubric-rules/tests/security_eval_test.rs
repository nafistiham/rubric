use rubric_core::{LintContext, Rule};
use rubric_rules::security::eval::Eval;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/security/eval/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/security/eval/clean.rb");

#[test]
fn detects_bare_eval() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Eval.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations (eval( and eval space)");
    assert!(diags.iter().all(|d| d.rule == "Security/Eval"));
}

#[test]
fn no_violation_for_safe_eval_variants() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = Eval.check_source(&ctx);
    assert_eq!(diags.len(), 0, "instance_eval/class_eval/module_eval should not be flagged");
}

#[test]
fn skips_eval_inside_string() {
    let source = "x = \"eval is dangerous\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = Eval.check_source(&ctx);
    assert_eq!(diags.len(), 0, "eval inside string should not be flagged");
}

#[test]
fn skips_comment_line() {
    let source = "# eval(something) is bad\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = Eval.check_source(&ctx);
    assert_eq!(diags.len(), 0, "eval in comment should not be flagged");
}

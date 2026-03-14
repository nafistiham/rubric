use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_elsif_condition::DuplicateElsifCondition;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/duplicate_elsif_condition/offending.rb");
const CLEAN: &str = include_str!("fixtures/lint/duplicate_elsif_condition/clean.rb");

#[test]
fn detects_duplicate_elsif() {
    let src = "if x > 0\n  a\nelsif x > 0\n  b\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateElsifCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for duplicate elsif");
    assert!(diags[0].message.contains("Duplicate"));
}

#[test]
fn no_violation_unique_conditions() {
    let src = "if x > 0\n  a\nelsif x < 0\n  b\nelse\n  c\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateElsifCondition.check_source(&ctx);
    assert!(diags.is_empty(), "unique conditions should not be flagged");
}

#[test]
fn no_violation_simple_if() {
    let src = "if x > 0\n  puts x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateElsifCondition.check_source(&ctx);
    assert!(diags.is_empty(), "simple if without elsif should be clean");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DuplicateElsifCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateElsifCondition"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = DuplicateElsifCondition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb");
}

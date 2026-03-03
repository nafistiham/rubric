use rubric_core::{LintContext, Rule};
use rubric_rules::style::mutable_constant::MutableConstant;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/mutable_constant/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/mutable_constant/corrected.rb");

#[test]
fn detects_mutable_constant() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MutableConstant.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/MutableConstant"));
}

#[test]
fn no_violation_for_frozen_constant() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MutableConstant.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// Setter method calls are not constant assignments — they must not be flagged.
#[test]
fn no_false_positive_for_setter_method_call() {
    let src = "Devise.mailer = 'Devise::Mailer'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for setter method call, got: {:?}",
        diags
    );
}

// A setter with a mutable RHS but a receiver before the dot is still not a
// constant assignment.
#[test]
fn no_false_positive_for_namespaced_setter() {
    let src = "Config.value = []\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for namespaced setter, got: {:?}",
        diags
    );
}

// Bare mutable array constant must still be detected.
#[test]
fn still_detects_mutable_array_constant() {
    let src = "NAMES = ['Alice', 'Bob']\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for mutable array constant, got none"
    );
    assert_eq!(diags[0].rule, "Style/MutableConstant");
}

// Bare mutable hash constant must still be detected.
#[test]
fn still_detects_mutable_hash_constant() {
    let src = "CONFIG = { a: 1 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for mutable hash constant, got none"
    );
    assert_eq!(diags[0].rule, "Style/MutableConstant");
}

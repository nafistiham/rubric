use rubric_core::{LintContext, Rule};
use rubric_rules::style::if_unless_modifier::IfUnlessModifier;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/if_unless_modifier/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/if_unless_modifier/corrected.rb");

#[test]
fn detects_if_block_that_could_be_modifier() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = IfUnlessModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/IfUnlessModifier"));
}

#[test]
fn no_violation_for_modifier_form() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = IfUnlessModifier.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

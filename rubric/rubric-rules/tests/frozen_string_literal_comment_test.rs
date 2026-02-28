use rubric_core::{LintContext, Rule};
use rubric_rules::style::frozen_string_literal_comment::FrozenStringLiteralComment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/frozen_string_literal_comment/offending.rb");
const CORRECTED: &str = include_str!("fixtures/style/frozen_string_literal_comment/corrected.rb");

#[test]
fn detects_missing_frozen_string_literal_comment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FrozenStringLiteralComment.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Style/FrozenStringLiteralComment"));
}

#[test]
fn no_violation_when_comment_present() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = FrozenStringLiteralComment.check_source(&ctx);
    assert!(diags.is_empty());
}

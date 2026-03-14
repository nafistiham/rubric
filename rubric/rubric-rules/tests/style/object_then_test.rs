use rubric_core::{LintContext, Rule};
use rubric_rules::style::object_then::ObjectThen;
use std::path::Path;

const OFFENDING: &str = include_str!("../fixtures/style/object_then/offending.rb");
const PASSING: &str = include_str!("../fixtures/style/object_then/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ObjectThen.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ObjectThen"));
}

#[test]
fn no_violation_on_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = ObjectThen.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}

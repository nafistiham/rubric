use rubric_core::{LintContext, Rule};
use rubric_rules::lint::ambiguous_regexp_literal::AmbiguousRegexpLiteral;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/ambiguous_regexp_literal/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AmbiguousRegexpLiteral.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/AmbiguousRegexpLiteral"));
}

#[test]
fn no_violation_on_clean() {
    let src = "p(/pattern/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = AmbiguousRegexpLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Division `a / b` (space after `/`) is NOT ambiguous — it's clearly arithmetic.
// Only `method /pattern/` (no space after `/`) is ambiguous.
#[test]
fn no_false_positive_for_division_with_space() {
    let src = "def ratio(a, b)\n  if a > b\n    a / b\n  else\n    b / a\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = AmbiguousRegexpLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "division `a / b` with space must not be flagged: {:?}", diags);
}

use rubric_core::{LintContext, Rule};
use rubric_rules::style::and_or::AndOr;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/and_or/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/and_or/corrected.rb");

#[test]
fn detects_and_or_keywords() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AndOr.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/AndOr"));
}

#[test]
fn no_violation_for_double_ampersand_or_pipe() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = AndOr.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// RuboCop `conditionals` style: `or raise` is idiomatic flow control — must NOT flag.
#[test]
fn no_false_positive_for_or_raise() {
    let source = "request.env['warden'] or raise MissingWarden\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or raise` is flow control and must not be flagged, got: {:?}",
        diags
    );
}

// RuboCop `conditionals` style: `or return` is idiomatic flow control — must NOT flag.
#[test]
fn no_false_positive_for_or_return() {
    let source = "find_user or return\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or return` is flow control and must not be flagged, got: {:?}",
        diags
    );
}

// RuboCop `conditionals` style: `and return` is idiomatic flow control — must NOT flag.
#[test]
fn no_false_positive_for_and_return() {
    let source = "valid? and return\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`and return` is flow control and must not be flagged, got: {:?}",
        diags
    );
}

// `and`/`or` used as boolean operators in a conditional must still be flagged.
#[test]
fn still_detects_and_in_conditional() {
    let source = "if a and b\n  do_thing\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "`and` as boolean operator in conditional must be flagged"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/AndOr"));
}

// `and`/`or` with additional flow-control keywords: `next` and `break` — must NOT flag.
#[test]
fn no_false_positive_for_or_next() {
    let source = "item or next\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or next` is flow control and must not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_and_break() {
    let source = "valid? and break\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`and break` is flow control and must not be flagged, got: {:?}",
        diags
    );
}

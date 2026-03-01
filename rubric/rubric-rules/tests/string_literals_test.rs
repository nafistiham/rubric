use rubric_core::LintContext;
use rubric_core::walker::walk;
use rubric_rules::style::string_literals::StringLiterals;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/string_literals/offending.rb");
const CORRECTED: &str = include_str!("fixtures/style/string_literals/corrected.rb");

#[test]
fn detects_double_quoted_strings() {
    let source = OFFENDING;
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let rules: Vec<Box<dyn rubric_core::Rule>> = vec![Box::new(StringLiterals)];
    let diags = walk(source.as_bytes(), &ctx, &rules);
    // "hello" and "world" should be flagged; "it's fine" and "has\nnewline" should not
    assert_eq!(diags.len(), 2, "expected 2 violations");
    assert!(diags.iter().all(|d| d.rule == "Style/StringLiterals"));
}

#[test]
fn no_violation_on_corrected() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let rules: Vec<Box<dyn rubric_core::Rule>> = vec![Box::new(StringLiterals)];
    let diags = walk(CORRECTED.as_bytes(), &ctx, &rules);
    assert!(diags.is_empty(), "expected no violations on corrected fixture, got: {:?}", diags);
}

#[test]
fn no_false_positive_on_interpolation_fragment() {
    // A child StringNode inside an InterpolatedStringNode may have content
    // containing "#{". Such nodes must not be flagged as missing single-quotes.
    let source = r#"x = "hello #{name} world""#;
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let rules: Vec<Box<dyn rubric_core::Rule>> = vec![Box::new(StringLiterals)];
    let diags = walk(source.as_bytes(), &ctx, &rules);
    assert!(
        diags.is_empty(),
        "interpolated string should not be flagged, got: {:?}",
        diags
    );
}

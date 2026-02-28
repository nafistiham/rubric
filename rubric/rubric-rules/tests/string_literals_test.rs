use rubric_core::LintContext;
use rubric_core::walker::walk;
use rubric_rules::style::string_literals::StringLiterals;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/string_literals/offending.rb");

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

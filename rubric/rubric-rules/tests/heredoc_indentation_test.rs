use rubric_core::{LintContext, Rule};
use rubric_rules::layout::heredoc_indentation::HeredocIndentation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/heredoc_indentation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/HeredocIndentation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "text = <<~RUBY\n  properly_indented\nRUBY\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// FP fix: <<~IDENTIFIER.method — identifier includes the chained method call in the
// raw text, but the terminator line is just the bare IDENTIFIER (e.g. `SQL`).
// Before the fix the rule extracted "SQL.squish" as the id and never found a matching
// terminator, causing it to scan past the heredoc body and flag subsequent lines.
#[test]
fn no_violation_on_squiggly_heredoc_with_chained_method() {
    let src = r#"def sql_query_string
  <<~SQL.squish
    SELECT *
    FROM accounts
  SQL
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for <<~SQL.squish with indented body; got: {:?}",
        diags
    );
}

// Variant: heredoc inside a method-call expression, so the `<<~` line ends with `)`.
// e.g.  Arel.sql(<<~SQL.squish)
#[test]
fn no_violation_on_squiggly_heredoc_with_method_and_closing_paren() {
    let src = "result = Arel.sql(\n  <<~SQL.squish\n    SELECT 1\n  SQL\n)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for <<~SQL.squish) with indented body; got: {:?}",
        diags
    );
}

// Variant: CSS squiggly heredoc chained with .squish
#[test]
fn no_violation_on_squiggly_css_heredoc_with_chained_method() {
    let src = "STYLES = <<~CSS.squish\n  height: 1em;\n  width: 1em;\nCSS\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for <<~CSS.squish; got: {:?}",
        diags
    );
}

// Quoted heredoc identifiers: <<~'SQL', <<~"SQL" — terminator is SQL (no quotes).
#[test]
fn no_violation_on_single_quoted_heredoc_identifier() {
    let src = "text = <<~'SQL'\n  SELECT 1\nSQL\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for <<~'SQL' with indented body; got: {:?}",
        diags
    );
}

#[test]
fn no_violation_on_double_quoted_heredoc_identifier() {
    let src = "text = <<~\"SQL\"\n  SELECT 1\nSQL\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for <<~\"SQL\" with indented body; got: {:?}",
        diags
    );
}

// Code after the heredoc closes must NOT be flagged.
#[test]
fn no_violation_on_unindented_code_after_heredoc_closes() {
    // The `end` and class closing `end` are at column 0 — they are not heredoc content
    // and must never be flagged.
    let src = "class Foo\n  def bar\n    <<~SQL.squish\n      SELECT 1\n    SQL\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations — unindented `end` lines after heredoc must not be flagged; got: {:?}",
        diags
    );
}

// Still detects genuine violations: body line starts at column 0 inside <<~.
#[test]
fn detects_genuinely_unindented_body_line() {
    // "no_indent" starts at col 0 inside a <<~ heredoc — that IS a violation.
    let src = "text = <<~RUBY\nno_indent\nRUBY\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for unindented content in <<~ heredoc"
    );
}

// Multiple heredocs in sequence — each one must be handled independently.
#[test]
fn no_violation_on_multiple_heredocs_with_chained_methods() {
    let src = "a = <<~SQL.squish\n  SELECT 1\nSQL\nb = <<~HTML.squish\n  <p>hi</p>\nHTML\ndef foo; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for multiple chained-method heredocs; got: {:?}",
        diags
    );
}

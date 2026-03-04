use rubric_core::{LintContext, Rule};
use rubric_rules::layout::first_argument_indentation::FirstArgumentIndentation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/first_argument_indentation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FirstArgumentIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/FirstArgumentIndentation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo(\n  bar,\n  baz\n)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstArgumentIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Heredoc body with SQL that starts a line with `(` should not be flagged.
// The `(` on the line after the heredoc opener is inside the heredoc body,
// not a method call argument list opener.
#[test]
fn no_violation_inside_squiggly_heredoc() {
    let src = concat!(
        "TEXT_SEARCH_RANKS = <<~SQL.squish\n",
        "  (\n",
        "      setweight(to_tsvector('simple', accounts.display_name), 'A') ||\n",
        "      setweight(to_tsvector('simple', accounts.username), 'B')\n",
        "  )\n",
        "SQL\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstArgumentIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations inside <<~SQL heredoc body, got: {:?}",
        diags
    );
}

// Heredoc body with lines ending in `(` should not trigger the rule.
#[test]
fn no_violation_inside_dash_heredoc() {
    let src = concat!(
        "query = <<-SQL\n",
        "  SELECT *\n",
        "  FROM (\n",
        "    SELECT id FROM accounts\n",
        "  )\n",
        "SQL\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstArgumentIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations inside <<-SQL heredoc body, got: {:?}",
        diags
    );
}

// Heredoc body with bare `<<WORD` (no sigil) should also be skipped.
#[test]
fn no_violation_inside_bare_heredoc() {
    let src = concat!(
        "msg = <<HEREDOC\n",
        "  foo(\n",
        "bar\n",
        "  )\n",
        "HEREDOC\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstArgumentIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations inside bare <<HEREDOC body, got: {:?}",
        diags
    );
}

// Code after a heredoc (on subsequent lines, past its terminator) must still
// be checked for real violations.
#[test]
fn violation_after_heredoc_still_detected() {
    let src = concat!(
        "msg = <<~TEXT\n",
        "  hello world\n",
        "TEXT\n",
        "foo(\n",
        "bar\n",
        ")\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstArgumentIndentation.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violation after heredoc to be detected"
    );
}

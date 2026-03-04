use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_method_call_indentation::MultilineMethodCallIndentation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/multiline_method_call_indentation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/multiline_method_call_indentation/corrected.rb");

#[test]
fn detects_trailing_dot_in_chained_call() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for trailing dots, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineMethodCallIndentation"));
}

#[test]
fn no_violation_with_leading_dots() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── Doc comments ending with period must NOT fire ──────────────────────────
// YARD-style `# description.` lines legitimately end with `.` (sentence end).
// The trailing-dot check must skip all comment lines.
#[test]
fn no_false_positive_for_doc_comment_ending_with_period() {
    let src = "##\n# Produces the name of a city.\n#\n# @return [String]\ndef city\n  fetch('city')\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "doc comment ending with period should not be flagged: {:?}", diags);
}

// ── Inline comments ending with period must NOT fire ───────────────────────
// `x = foo # Gets the value.` — the `.` is inside the comment, not in code.
#[test]
fn no_false_positive_for_inline_comment_ending_with_period() {
    let src = "x = foo # Gets the value.\ny = bar\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "inline comment ending with period should not be flagged: {:?}", diags);
}

// ── True positive: code line with trailing dot still fires ─────────────────
#[test]
fn still_detects_trailing_dot_code_line() {
    let src = "result = foo.\n  bar\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "trailing dot in code should still be flagged");
}

// ── Heredoc body lines ending with `.` must NOT fire ───────────────────────
// Text inside <<~HEREDOC ... HEREDOC is plain string content, not code.
// Sentences ending with periods are extremely common there.
#[test]
fn no_false_positive_inside_squiggly_heredoc_body() {
    let src = concat!(
        "abort <<~MESSAGE\n",
        "  Please fix your configuration.\n",
        "  See the documentation.\n",
        "MESSAGE\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "heredoc body lines ending with period should not be flagged: {:?}",
        diags
    );
}

// ── Dash-heredoc body lines ending with `.` must NOT fire ──────────────────
#[test]
fn no_false_positive_inside_dash_heredoc_body() {
    let src = concat!(
        "long_desc <<-LONG_DESC\n",
        "  Generate and broadcast new RSA keys as part of security\n",
        "  maintenance.\n",
        "LONG_DESC\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "dash-heredoc body lines ending with period should not be flagged: {:?}",
        diags
    );
}

// ── Heredoc with inline comment on opener must NOT fire inside body ─────────
// `abort <<~ERROR # rubocop:disable Rails/Exit`  — the body follows.
#[test]
fn no_false_positive_heredoc_with_inline_comment_on_opener() {
    let src = concat!(
        "abort <<~ERROR # rubocop:disable Rails/Exit\n",
        "  The RAILS_ENV environment variable is not set.\n",
        "\n",
        "  Please set it correctly.\n",
        "ERROR\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "heredoc body after opener with inline comment should not be flagged: {:?}",
        diags
    );
}

// ── Multi-line regex literal body must NOT fire ─────────────────────────────
// A line like `      (\.` inside a `/.../.../` regex is not a method call.
#[test]
fn no_false_positive_inside_multiline_regex() {
    let src = concat!(
        "EXPR = /\n",
        "  \\{\\{\n",
        "    [a-z_]+\n",
        "    (\\.\n",
        "      ([a-z_]+|[0-9]+)\n",
        "    )*\n",
        "  \\}\\}\n",
        "/iox\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "lines inside a multi-line regex ending with dot should not be flagged: {:?}",
        diags
    );
}

// ── Heredoc followed by chained code: body skipped, real dot still fires ────
// `<<~SQL.squish` — the BODY is skipped, but a REAL trailing dot AFTER the
// heredoc on a subsequent code line must still be detected.
#[test]
fn heredoc_body_skipped_but_subsequent_real_dot_flagged() {
    let src = concat!(
        "execute <<~SQL\n",
        "  SELECT 1.\n",
        "SQL\n",
        "result = foo.\n",
        "  bar\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    // The heredoc body line (`  SELECT 1.`) must NOT be flagged.
    // The code line (`result = foo.`) MUST be flagged.
    assert_eq!(
        diags.len(),
        1,
        "only the real trailing-dot code line after the heredoc should fire: {:?}",
        diags
    );
}

// ── Quoted-identifier heredoc (`<<~'TERM'`) must also be handled ────────────
#[test]
fn no_false_positive_inside_single_quoted_heredoc() {
    let src = concat!(
        "msg = <<~'END'\n",
        "  No interpolation here.\n",
        "  Plain text with a dot.\n",
        "END\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "single-quoted heredoc body should not be flagged: {:?}",
        diags
    );
}

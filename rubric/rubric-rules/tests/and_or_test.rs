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

// `and`/`or` inside a double-quoted string literal must NOT be flagged.
#[test]
fn no_false_positive_and_inside_double_quoted_string() {
    let source = r#"logger.info "Upgrade for more features and support""# ;
    let source = format!("{}\n", source);
    let ctx = LintContext::new(Path::new("test.rb"), &source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`and` inside string literal must not be flagged, got: {:?}",
        diags
    );
}

// `or` inside a double-quoted string literal must NOT be flagged.
#[test]
fn no_false_positive_or_inside_double_quoted_string() {
    let source = "msg = \"Ensure Redis is running in the same AZ or datacenter\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or` inside string literal must not be flagged, got: {:?}",
        diags
    );
}

// `and`/`or` inside single-quoted strings must NOT be flagged.
#[test]
fn no_false_positive_and_inside_single_quoted_string() {
    let source = "msg = 'features and support'\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`and` inside single-quoted string literal must not be flagged, got: {:?}",
        diags
    );
}

// `and`/`or` inside a heredoc body must NOT be flagged.
#[test]
fn no_false_positive_or_inside_heredoc_body() {
    let source = "msg = <<~TEXT\n  Ensure Redis is running in the same AZ or datacenter\nTEXT\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or` inside heredoc body must not be flagged, got: {:?}",
        diags
    );
}

// Real code using `and`/`or` as operators must still be flagged even when strings are present.
#[test]
fn still_detects_and_outside_string_when_string_also_present() {
    let source = "x = foo and bar\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "`and` as boolean operator must be flagged even when strings present"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/AndOr"));
}

// `or` inside an inline comment after code must NOT be flagged.
// Example: `field :token  # Only if unlock strategy is :email or :both`
#[test]
fn no_false_positive_or_inside_inline_comment() {
    let source = "field :unlock_token, type: String # Only if unlock strategy is :email or :both\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or` inside inline comment must not be flagged, got: {:?}",
        diags
    );
}

// `and` inside an inline comment after code must NOT be flagged.
// Example: `# 240.0.0.0 - 255.255.255.254  and  255.255.255.255`
#[test]
fn no_false_positive_and_inside_inline_comment() {
    let source = "/^(24\\d|25[0-5])\\./     # 240.0.0.0 - 255.255.255.254  and  255.255.255.255\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`and` inside inline comment must not be flagged, got: {:?}",
        diags
    );
}

// `or` inside a regex literal must NOT be flagged.
// Example: `assert_match(/\AValid cards can be left blank or include/, e.message)`
#[test]
fn no_false_positive_or_inside_regex_literal() {
    let source = "assert_match(/\\AValid credit cards argument can be left blank or include/, e.message)\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or` inside regex literal must not be flagged, got: {:?}",
        diags
    );
}

// `or` inside a `%q(...)` percent literal must NOT be flagged.
// Example: `raise %q(You need ActiveRecord >= 7.2 or to add gem)`
#[test]
fn no_false_positive_or_inside_percent_q_literal() {
    let source = "raise %q(You need ActiveRecord >= 7.2 or to add gem to your Gemfile)\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or` inside %q(...) literal must not be flagged, got: {:?}",
        diags
    );
}

// `or` inside a `%(...)` percent literal (single line with \n escapes) must NOT be flagged.
// Matches the real pattern from devise: result = %(... sign in or sign up ...)
#[test]
fn no_false_positive_or_inside_bare_percent_literal() {
    // Simulates the actual devise line where the whole %() is on one line.
    let source = "result = %(<?xml version=\"1.0\"?>\\n<error>sign in or sign up</error>\\n)\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`or` inside %(...) literal must not be flagged, got: {:?}",
        diags
    );
}

// Code followed by a comment: code-`or`/`and` must still be flagged,
// comment-`or`/`and` must not.
#[test]
fn flags_code_or_but_not_comment_or_on_same_line() {
    // `a or b` in code part, then `# or something` in comment — only one flag.
    let source = "x = a or b  # use or for flow control\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = AndOr.check_source(&ctx);
    // The code `or` is genuine; the comment `or` must be skipped.
    assert_eq!(
        diags.len(),
        1,
        "expected exactly 1 violation (code `or`, not comment `or`), got: {:?}",
        diags
    );
}

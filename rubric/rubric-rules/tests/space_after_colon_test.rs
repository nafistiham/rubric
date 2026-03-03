use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_after_colon::SpaceAfterColon;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_after_colon/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_after_colon/corrected.rb");

#[test]
fn detects_missing_space_after_colon() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAfterColon"));
}

#[test]
fn no_violation_with_space_after_colon() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── False positive: POSIX character class closing `:` (e.g., `[:word:]`) ────
#[test]
fn no_false_positive_for_posix_char_class_in_regex() {
    let src = "names = str.scan(/[[:word:]]+/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "POSIX char class [:word:] falsely flagged: {:?}", diags);
}

// ── False positive: `[:word:]` inside %r{} percent regex ────────────────────
#[test]
fn no_false_positive_for_posix_char_class_in_percent_regex() {
    let src = "RE = %r{(?<![=/[:word:]])@foo}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "POSIX char class [:word:] in %r{{}} falsely flagged: {:?}", diags);
}

// ── False positive: keyword argument shorthand `name:,` / `cursor:)` ─────────
#[test]
fn no_false_positive_for_keyword_argument_shorthand() {
    let src = "x = Foo.new(name:, size:, latency:)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "keyword arg shorthand falsely flagged: {:?}", diags);
}

// ── False positive: required keyword arg in method def `def foo(cursor:)` ────
#[test]
fn no_false_positive_for_required_keyword_arg() {
    let src = "def rows(cursor:)\n  @data\nend\ndef batches(cursor:, batch_size: 100)\n  @data\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "required keyword arg falsely flagged: {:?}", diags);
}

// ── False positive: keyword shorthands in hash literal `{name:, size:}` ──────
#[test]
fn no_false_positive_for_keyword_shorthand_in_hash() {
    let src = "h = {name:, size:, type: :key, code:, modifiers:}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "keyword shorthand in hash falsely flagged: {:?}", diags);
}

// ── False positive: URL scheme `://` should not be flagged ────────────────────
#[test]
fn no_false_positive_for_url_scheme() {
    let src = "url = \"https://example.com/path\"\nredis = \"redis://host:6379/0\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "URL scheme :// falsely flagged: {:?}", diags);
}

// ── False positive: colon inside %r!...! percent regex ───────────────────────
#[test]
fn no_false_positive_for_colon_in_percent_regex() {
    let src = "m = %r!http://127.0.0.1:(\\d+)!.match(str)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "colon in %r!...! percent regex falsely flagged: {:?}", diags);
}

// ── False positive: colon inside standard regex `/pattern:/` ─────────────────
#[test]
fn no_false_positive_for_colon_in_regex() {
    let src = "result_str.gsub!(/\\A(Failure:|Error:)\\s/, '\\1 ')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "colon in /regex/ falsely flagged: {:?}", diags);
}

// ── False positive: colon inside double-quoted string ────────────────────────
#[test]
fn no_false_positive_for_colon_in_string() {
    let src = "url = \"http://host:8080/path\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "colon in string falsely flagged: {:?}", diags);
}

// ── False positive: colon inside heredoc body ────────────────────────────────
#[test]
fn no_false_positive_for_colon_in_heredoc() {
    let src = "req = <<~REQ\n  X:POST / HTTP/1.1\n  Host: example.com:456\nREQ\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(diags.is_empty(), "colon in heredoc body falsely flagged: {:?}", diags);
}

// ── Still detects missing space in hash literal ───────────────────────────────
#[test]
fn still_detects_missing_space_in_hash() {
    let src = "x = {a:1, b:2}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAfterColon.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for {{a:1}}, got none");
    assert_eq!(diags.len(), 2, "expected 2 violations, got: {:?}", diags);
}

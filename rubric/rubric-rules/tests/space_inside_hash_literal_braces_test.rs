use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_inside_hash_literal_braces::SpaceInsideHashLiteralBraces;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_inside_hash_literal_braces/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_inside_hash_literal_braces/corrected.rb");

#[test]
fn detects_missing_space_inside_hash_literal_braces() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceInsideHashLiteralBraces.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceInsideHashLiteralBraces"));
}

#[test]
fn no_violation_with_space_inside_hash_literal_braces() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceInsideHashLiteralBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_for_regex_quantifiers() {
    let src = "x = str.gsub(/(\\.d{2})(\\.d{7})/, '\\\\1-\\\\2')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideHashLiteralBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in regex quantifiers, got: {:?}", diags);
}

#[test]
fn no_violation_for_percent_r_regex() {
    let src = "MENTION_RE = %r{(?<![=/[:word:]])@(([a-z0-9]+)(?:@[[:word:]]+)?)}i\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideHashLiteralBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in %r{{}} regex, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_braces_in_multiline_regex_body() {
    // \{\{ and \}\} inside a multiline /regex/iox body are NOT hash braces
    let src = concat!(
        "RENDER_RE = /\n",
        "  \\{\\{[^}]+\\}\\}\n",
        "/iox\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideHashLiteralBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in multiline regex body, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_closing_brace_of_multiline_percent_regex() {
    // The closing } on its own line closes %r{...} — not a hash literal brace
    let src = concat!(
        "URL_RE = %r{\n",
        "  (?:https?://)\n",
        "  ([a-z]+\\.)+[a-z]+\n",
        "}iox\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInsideHashLiteralBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for multiline %r{{}} close, got: {:?}", diags);
}

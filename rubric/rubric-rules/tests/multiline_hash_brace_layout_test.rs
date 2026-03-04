use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_hash_brace_layout::MultilineHashBraceLayout;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/multiline_hash_brace_layout/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineHashBraceLayout"));
}

#[test]
fn no_violation_on_clean() {
    let src = "h = {\n  a: 1,\n  b: 2\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// FP: single-line block ending with `}` — balanced braces, not a multiline hash closer
// e.g. `format.all { super(**) }` after a multiline hash on a previous line
#[test]
fn no_false_positive_for_single_line_block_after_multiline_hash() {
    let src = concat!(
        "format.json do\n",
        "  render json: {\n",
        "    redirect_to: path,\n",
        "  }, status: 200\n",
        "end\n",
        "format.all { super(**) }\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "single-line block after multiline hash falsely flagged: {:?}", diags);
}

// FP: empty hash literal `@headers = {}` on its own line
#[test]
fn no_false_positive_for_empty_hash_literal() {
    let src = concat!(
        "def initialize\n",
        "  @options = {\n",
        "    follow: { max_hops: 3 },\n",
        "  }.merge(options)\n",
        "  @headers = {}\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "empty hash `@headers = {{}}` falsely flagged: {:?}", diags);
}

// FP: stabby lambda `-> { expr }` on a line ending with `}`
#[test]
fn no_false_positive_for_stabby_lambda_single_line() {
    let src = concat!(
        "scope :foo, lambda { |x|\n",
        "  x.bar\n",
        "}\n",
        "scope :baz, -> { where(active: true) }\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    // Only the multiline lambda `{ ... }` is a potential issue — but rubocop wouldn't flag it
    // either. The single-line `-> { ... }` must NOT be flagged.
    let flagged_lines: Vec<_> = diags.iter().collect();
    // The stabby lambda line has balanced braces and must produce 0 diagnostics.
    assert!(
        diags.is_empty(),
        "stabby lambda `-> {{ ... }}` falsely flagged: {:?}", flagged_lines
    );
}

// FP: `#{interpolation}` inside heredoc ending with `}`
#[test]
fn no_false_positive_for_string_interpolation_ending_with_brace() {
    let src = concat!(
        "let(:html) { <<~HTML }\n",
        "  <script>\n",
        "    #{ld_json}\n",
        "  </script>\n",
        "HTML\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "interpolation `#{{ld_json}}` in heredoc falsely flagged: {:?}", diags);
}

// FP: `\}\}` inside a multiline regex body — backslash-escaped braces
#[test]
fn no_false_positive_for_escaped_braces_in_regex() {
    let src = concat!(
        "REGEXP = /\n",
        "  \\{\\{\n",
        "    [a-z]+\n",
        "  \\}\\}\n",
        "/iox\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineHashBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "escaped braces in regex falsely flagged: {:?}", diags);
}

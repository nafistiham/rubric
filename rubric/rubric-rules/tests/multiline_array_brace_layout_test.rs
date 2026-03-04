use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_array_brace_layout::MultilineArrayBraceLayout;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/multiline_array_brace_layout/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineArrayBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineArrayBraceLayout"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = [\n  1,\n  2,\n  3\n]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineArrayBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Element ending with `]` from an inner %w[] literal should not be flagged
// as the closing bracket of the outer multiline array
#[test]
fn no_false_positive_for_inner_percent_w_closing_bracket() {
    let src = concat!(
        "        char_range = [\n",
        "          Array('0'..'9'),\n",
        "          Array('a'..'z'),\n",
        "          urlsafe ? %w[- _] : %w[+ /]\n",
        "        ].flatten\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineArrayBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "inner %w[] bracket falsely flagged: {:?}", diags);
}

// A `]` that appears AFTER a comment line ending with `[` must not be flagged.
// The backward scan must skip comment lines and not treat them as array openers.
#[test]
fn no_false_positive_for_comment_ending_with_open_bracket() {
    // Comment line ends with `[` — the backward scan used to find this and set
    // is_multiline=true, causing the next `]`-containing line to be flagged.
    let src = concat!(
        "# Example usage: foo[\n",
        "result = some_method.call]\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineArrayBraceLayout.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "comment line ending with [ must not be treated as array opener: {:?}",
        diags
    );
}

// Comment lines that look like array-open must be skipped during back-scan
#[test]
fn no_false_positive_for_comment_before_real_array_close() {
    // Real multiline array opening is on a non-comment line above a comment
    let src = concat!(
        "x = [\n",
        "  # #=> [foo, bar]\n",
        "  1,\n",
        "  2\n",
        "]\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineArrayBraceLayout.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "comment line with bracket inside multiline array should not cause FP: {:?}",
        diags
    );
}

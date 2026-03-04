use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_method_call_brace_layout::MultilineMethodCallBraceLayout;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/multiline_method_call_brace_layout/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineMethodCallBraceLayout"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo(\n  bar,\n  baz\n)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// ── False positive: symmetrical style — opening ( has first arg on same line ─
// `foo(arg1,\n    arg2)` is valid `symmetrical` style. Must not fire.
#[test]
fn no_false_positive_for_symmetrical_style() {
    let src = "foo(arg1,\n    arg2)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "symmetrical-style call falsely flagged: {:?}", diags);
}

// ── False positive: method call with parens in string literal on prior line ──
#[test]
fn no_false_positive_for_paren_in_string() {
    let src = "puts \"some (text)\"\nresult = foo(x)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "paren in string on prior line falsely flagged: {:?}", diags);
}

// ── True positive: bare opening `(` means ) must be on own line ──────────────
#[test]
fn detects_bare_open_paren_style_violation() {
    let src = "foo(\n  arg1,\n  arg2)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "bare-open-paren style with ) on arg line should be flagged");
}

// ── FP: parentheses balanced on the same line — not a multiline closer ────────
// e.g. `(supplemental ? translate('...') : [])` — both `(` and `)` on same line
#[test]
fn no_false_positive_for_balanced_parens_on_line() {
    let src = concat!(
        "word_list = (\n",
        "  translate('faker.lorem.words') +\n",
        "  (supplemental ? translate('faker.lorem.supplemental') : [])\n",
        ")\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "balanced parens on line falsely flagged: {:?}", diags);
}

// ── FP: commented-out code — line starting with `#` ──────────────────────────
#[test]
fn no_false_positive_for_comment_line() {
    let src = concat!(
        "# ApplicationController.renderer.defaults.merge!(\n",
        "#   http_host: 'example.org',\n",
        "#   https: false\n",
        "# )\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "commented-out `)` falsely flagged: {:?}", diags);
}

// ── FP: `})` closing both a hash arg and the method call — symmetrical style ──
// `expect(subject).to include({...})` — rubocop symmetrical style allows this
#[test]
fn no_false_positive_for_hash_arg_closing_paren() {
    let src = concat!(
        "expect(subject).to include({\n",
        "  'type' => 'Create',\n",
        "  'actor' => tag_manager.uri_for(account),\n",
        "})\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodCallBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "`}}` before `)` falsely flagged: {:?}", diags);
}

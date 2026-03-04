use rubric_core::{LintContext, Rule};
use rubric_rules::style::empty_method::EmptyMethod;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/empty_method/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyMethod.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/EmptyMethod"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyMethod.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Regression: single-line def followed by class/module `end` must not be flagged
#[test]
fn no_fp_single_line_def_before_class_end() {
    let src = "class Foo\n  def show; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyMethod.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "single-line def followed by class end must not be flagged, got: {:?}",
        diags
    );
}

// Regression: endless method (def foo = expr) followed by `end` must not be flagged
#[test]
fn no_fp_endless_method_before_class_end() {
    let src = "class Foo\n  def text_for_warning = text\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyMethod.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "endless method followed by class end must not be flagged, got: {:?}",
        diags
    );
}

// Regression: def down; end at end of class (single-line form, already correct)
#[test]
fn no_fp_single_line_def_down_end() {
    let src = "class Foo\n  def up\n    do_work\n  end\n\n  def down; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyMethod.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def down; end (single-line) must not be flagged, got: {:?}",
        diags
    );
}

// A genuine empty multi-line method should still be flagged
#[test]
fn flags_genuine_empty_multiline_method() {
    let src = "class Foo\n  def show\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyMethod.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "genuine empty multi-line def should be flagged"
    );
}

// Method with default args and empty body should still be flagged
#[test]
fn flags_empty_method_with_default_args() {
    let src = "class Foo\n  def bar(x = 1)\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyMethod.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "empty method with default args should be flagged, got: {:?}",
        diags
    );
}

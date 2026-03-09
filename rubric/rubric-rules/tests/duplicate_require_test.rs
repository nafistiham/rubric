use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_require::DuplicateRequire;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/duplicate_require/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DuplicateRequire.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateRequire"));
}

#[test]
fn no_violation_on_clean() {
    let src = "require 'foo'\nrequire 'bar'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateRequire.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// require vs require_relative must not be conflated

#[test]
fn no_false_positive_require_vs_require_relative() {
    // `require 'foo'` and `require_relative 'foo'` load different files.
    let src = "require_relative 'helper'\nrequire 'helper'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateRequire.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "require and require_relative with same path must not be flagged; got: {:?}",
        diags
    );
}

#[test]
fn still_detects_duplicate_require() {
    let src = "require 'foo'\nrequire 'bar'\nrequire 'foo'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateRequire.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected exactly one duplicate; got: {:?}", diags);
}

#[test]
fn still_detects_duplicate_require_relative() {
    let src = "require_relative 'helper'\nrequire_relative 'helper'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateRequire.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected exactly one duplicate; got: {:?}", diags);
}

// Heredoc body skipping

#[test]
fn no_false_positive_require_in_heredoc_body() {
    // `require` inside a <<~RUBY heredoc is template content, not a real require.
    let src = concat!(
        "require 'rails/all'\n",
        "\n",
        "template = <<~RUBY\n",
        "  require 'rails/all'\n",
        "  Bundler.require(*Rails.groups)\n",
        "RUBY\n",
        "\n",
        "puts template\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateRequire.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "require in heredoc body must not be counted; got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_require_in_dash_heredoc_body() {
    let src = concat!(
        "require 'active_record'\n",
        "content = <<-RUBY\n",
        "  require 'active_record'\n",
        "RUBY\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateRequire.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "require in <<- heredoc body must not be counted; got: {:?}",
        diags
    );
}

#[test]
fn still_detects_duplicate_after_heredoc() {
    // A real duplicate appearing after a heredoc must still be flagged.
    let src = concat!(
        "require 'foo'\n",
        "x = <<~RUBY\n",
        "  require 'bar'\n",
        "RUBY\n",
        "require 'foo'\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateRequire.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected one duplicate after heredoc; got: {:?}", diags);
}

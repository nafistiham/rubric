use rubric_core::{LintContext, Rule};
use rubric_rules::lint::debugger_statement::DebuggerStatement;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/debugger_statement/offending.rb");
const CLEAN: &str = include_str!("fixtures/lint/debugger_statement/clean.rb");

#[test]
fn detects_debugger_statements() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/DebuggerStatement"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_binding_pry() {
    let src = "def foo\n  binding.pry\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(!diags.is_empty(), "binding.pry should be flagged");
    assert!(diags[0].message.contains("binding.pry"));
}

#[test]
fn detects_byebug() {
    let src = "def foo\n  byebug\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(!diags.is_empty(), "byebug should be flagged");
    assert!(diags[0].message.contains("byebug"));
}

#[test]
fn detects_debugger() {
    let src = "debugger\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(!diags.is_empty(), "debugger should be flagged");
}

#[test]
fn does_not_flag_commented_out() {
    let src = "def foo\n  # byebug\n  # binding.pry\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(diags.is_empty(), "commented debugger calls should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_inside_string() {
    let src = r#"msg = "call byebug here"
other = 'use binding.pry'
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(diags.is_empty(), "debugger calls inside strings should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_identifier_containing_byebug() {
    // e.g. a method named `my_byebug_helper` should not be flagged
    let src = "def my_byebug_helper\n  puts 'hi'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(diags.is_empty(), "identifier containing 'byebug' should not be flagged, got: {:?}", diags);
}

#[test]
fn skips_heredoc_body() {
    let src = "code = <<~RUBY\n  byebug\nRUBY\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(diags.is_empty(), "heredoc body should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_pry_start() {
    let src = "Pry.start\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(!diags.is_empty(), "Pry.start should be flagged");
}

#[test]
fn detects_binding_break() {
    let src = "binding.break\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DebuggerStatement.check_source(&ctx);
    assert!(!diags.is_empty(), "binding.break should be flagged");
}

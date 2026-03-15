use rubric_core::{LintContext, Rule};
use rubric_rules::style::global_std_stream::GlobalStdStream;
use std::path::Path;

fn check(src: &str) -> Vec<rubric_core::Diagnostic> {
    let ctx = LintContext::new(Path::new("test.rb"), src);
    GlobalStdStream.check_source(&ctx)
}

#[test]
fn flags_dollar_stdout_puts() {
    let diags = check("$stdout.puts \"hello\"\n");
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("STDOUT"));
    assert!(diags[0].message.contains("$stdout"));
}

#[test]
fn flags_dollar_stderr_puts() {
    let diags = check("$stderr.puts \"error\"\n");
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("STDERR"));
    assert!(diags[0].message.contains("$stderr"));
}

#[test]
fn flags_dollar_stdout_write() {
    let diags = check("$stdout.write(data)\n");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule, "Style/GlobalStdStream");
}

#[test]
fn no_violation_for_stdout_constant() {
    let diags = check("STDOUT.puts \"hello\"\n");
    assert!(diags.is_empty(), "STDOUT constant should not be flagged");
}

#[test]
fn no_violation_for_stderr_constant() {
    let diags = check("STDERR.puts \"error\"\n");
    assert!(diags.is_empty(), "STDERR constant should not be flagged");
}

#[test]
fn no_violation_in_string() {
    let diags = check("msg = \"$stdout is the standard output\"\n");
    assert!(diags.is_empty(), "$stdout inside a string should not be flagged");
}

#[test]
fn no_violation_in_comment() {
    let diags = check("# $stdout.puts \"hello\"\n");
    assert!(diags.is_empty(), "$stdout inside a comment should not be flagged");
}

#[test]
fn no_violation_in_single_quoted_string() {
    let diags = check("msg = '$stderr'\n");
    assert!(diags.is_empty(), "$stderr inside a single-quoted string should not be flagged");
}

#[test]
fn flags_multiple_on_same_file() {
    let src = "$stdout.puts \"hello\"\n$stderr.puts \"error\"\n";
    let diags = check(src);
    assert_eq!(diags.len(), 2);
}

#[test]
fn flags_dollar_stdout_inline_in_expression() {
    let diags = check("io = $stdout\n");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule, "Style/GlobalStdStream");
}

#[test]
fn rule_name_is_correct() {
    assert_eq!(GlobalStdStream.name(), "Style/GlobalStdStream");
}

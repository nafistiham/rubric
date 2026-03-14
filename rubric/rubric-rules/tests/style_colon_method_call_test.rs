use rubric_core::{LintContext, Rule};
use rubric_rules::style::colon_method_call::ColonMethodCall;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/colon_method_call/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/colon_method_call/clean.rb");

#[test]
fn detects_kernel_colon_puts() {
    let src = "Kernel::puts \"hello\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for Kernel::puts");
    assert_eq!(diags[0].rule, "Style/ColonMethodCall");
    assert!(diags[0].message.contains("::"));
}

#[test]
fn detects_file_colon_join() {
    let src = "File::join(\"a\", \"b\")\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for File::join");
}

#[test]
fn no_violation_for_dot_call() {
    let src = "Kernel.puts \"hello\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(diags.is_empty(), "Kernel.puts should not be flagged");
}

#[test]
fn no_violation_for_constant_access() {
    let src = "x = Foo::BAR\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(diags.is_empty(), "Foo::BAR constant access should not be flagged");
}

#[test]
fn no_violation_for_module_namespace() {
    let src = "class Foo::Bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(diags.is_empty(), "Foo::Bar module namespace should not be flagged");
}

#[test]
fn no_violation_in_comment() {
    let src = "# Kernel::puts is bad\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(diags.is_empty(), ":: in comment should not be flagged");
}

#[test]
fn no_violation_in_string() {
    let src = r#"msg = "use Kernel::puts here"
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(diags.is_empty(), ":: in string should not be flagged, got: {:?}", diags);
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ColonMethodCall"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = ColonMethodCall.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb, got: {:?}", diags);
}

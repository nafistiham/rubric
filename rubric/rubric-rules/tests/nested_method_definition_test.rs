use rubric_core::{LintContext, Rule};
use rubric_rules::lint::nested_method_definition::NestedMethodDefinition;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/nested_method_definition/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/NestedMethodDefinition"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  bar\nend\n\ndef bar\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Endless method false-positive tests

#[test]
fn no_false_positive_for_endless_method_in_class() {
    // An endless method (`def foo = expr`) has no matching `end`.
    // A regular method following it must NOT be flagged as nested.
    let src = "class Foo\n  def set_class = Something\n  def bar\n    1\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations; got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_module_with_endless_methods() {
    // Multiple endless methods in a module must not accumulate phantom depth.
    let src =
        "module Router\n  def head(path, &) = route(:head, path, &)\n  def get(path, &) = route(:get, path, &)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations; got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_endless_method_with_equals_in_body() {
    // Endless method whose body contains `==` must not be mis-detected.
    let src = "class Foo\n  def enabled? = flags == :on\n  def bar\n    2\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations; got: {:?}",
        diags
    );
}

#[test]
fn still_detects_genuine_nested_def() {
    // A real `def` inside a `def` body must still be flagged.
    let src = "def outer\n  def inner\n    1\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violation for genuine nested def"
    );
    assert_eq!(diags.len(), 1);
}

#[test]
fn still_detects_nested_def_after_endless_in_outer() {
    // Endless method at top level followed by a normal def containing a nested def.
    let src = "def simple = 1\ndef outer\n  def inner\n    2\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violation for nested def after top-level endless method"
    );
    assert_eq!(diags.len(), 1);
}

// Heredoc false-positive tests

#[test]
fn no_false_positive_for_def_inside_squiggly_heredoc() {
    // `def` appearing inside a <<~WORD heredoc string must not be flagged.
    let src = concat!(
        "def assert_something!(x)\n",
        "  raise ArgumentError, <<~MSG\n",
        "    Example:\n",
        "      def build(params, cursor:)\n",
        "        1\n",
        "      end\n",
        "  MSG\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations in heredoc body; got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_def_inside_dash_heredoc() {
    // `def` appearing inside a <<-WORD heredoc (indented terminator) must not be flagged.
    let src = concat!(
        "def render_template\n",
        "  class_eval <<-RUBY, __FILE__, __LINE__\n",
        "    def _erb_content\n",
        "      'hello'\n",
        "    end\n",
        "  RUBY\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations in heredoc body; got: {:?}",
        diags
    );
}

use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_methods::DuplicateMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/duplicate_methods/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateMethods"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  1\nend\n\ndef bar\n  2\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

/// Methods with the same name in different nested classes must not be flagged.
/// Pattern: class Outer > class Inner > def initialize; end; def initialize (outer); end
#[test]
fn no_fp_nested_class_same_method_name() {
    let src = r#"
class Outer
  class Inner
    def initialize(x)
      @x = x
    end
  end

  def initialize(y)
    @y = y
  end
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations: initialize in Inner vs Outer are different scopes, got: {:?}",
        diags
    );
}

/// Methods with the same name in sibling nested classes must not be flagged.
#[test]
fn no_fp_sibling_nested_classes() {
    let src = r#"
class Container
  class Foo
    def call
      "foo"
    end
  end

  class Bar
    def call
      "bar"
    end
  end
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations: call in Foo vs Bar are separate class scopes, got: {:?}",
        diags
    );
}

/// class << self creates a separate scope — class methods must not conflict
/// with instance methods of the same name.
#[test]
fn no_fp_class_self_vs_instance_method() {
    let src = r#"
class Request
  class << self
    def http_client
      HTTP.use(:auto_inflate)
    end
  end

  def http_client
    @http_client ||= Request.http_client
  end
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations: http_client in class<<self vs instance are different scopes, got: {:?}",
        diags
    );
}

/// RSpec helper defs inside separate context/describe do-blocks are not duplicates.
#[test]
fn no_fp_rspec_helper_defs_in_different_context_blocks() {
    let src = r#"
RSpec.describe Foo do
  context 'case A' do
    def helper
      "a"
    end
  end

  context 'case B' do
    def helper
      "b"
    end
  end
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations: helper in separate context blocks, got: {:?}",
        diags
    );
}

/// def inside a heredoc body must not be flagged as a duplicate.
#[test]
fn no_fp_def_inside_heredoc_body() {
    let src = r#"
class Foo
  def real_method
    raise ArgumentError, <<~MSG
      Example usage:
        def real_method(x)
          x + 1
        end
    MSG
  end
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations: def inside heredoc body is not a real definition, got: {:?}",
        diags
    );
}

/// Singleton method definitions on object variables (def obj.method) should not
/// be flagged as duplicates of each other or of same-named class methods.
#[test]
fn no_fp_singleton_method_on_variable() {
    let src = r#"
class MyTest
  def test_one
    obj = Object.new
    def obj.call
      1
    end
  end

  def test_two
    obj = Object.new
    def obj.call
      2
    end
  end
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations: singleton methods on different variable instances, got: {:?}",
        diags
    );
}

/// A genuine duplicate within the same class must still be detected.
#[test]
fn detects_real_duplicate_in_class() {
    let src = r#"
class Foo
  def bar
    1
  end

  def bar
    2
  end
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for duplicate bar in the same class"
    );
    assert!(diags.iter().any(|d| d.message.contains("bar")));
}

/// A genuine duplicate at file scope must still be detected.
#[test]
fn detects_real_duplicate_at_file_scope() {
    let src = "def foo\n  1\nend\n\ndef foo\n  2\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for duplicate foo at file scope"
    );
}

/// `end` as a word inside a string literal must not corrupt the frame stack.
#[test]
fn no_fp_end_word_in_string_literal() {
    // `"unexpected end of input"` contains `end` as a word — count_ends must not
    // count it, otherwise the def frame is popped early and subsequent methods
    // leak into the wrong scope.
    let src = concat!(
        "class Parser\n",
        "  def tokenize(input)\n",
        "    raise \"unexpected end of input\" if input.empty?\n",
        "    input.chars\n",
        "  end\n",
        "\n",
        "  def parse(tokens)\n",
        "    tokens.first\n",
        "  end\n",
        "end\n",
        "\n",
        "class Lexer\n",
        "  def parse(src)\n",
        "    src\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "end in string must not corrupt frame stack; got: {:?}",
        diags
    );
}

/// `end` appearing after a `#` inline comment must not be counted.
#[test]
fn no_fp_end_word_in_inline_comment() {
    let src = concat!(
        "class Foo\n",
        "  def bar\n",
        "    do_thing # this is the end of the process\n",
        "  end\n",
        "end\n",
        "\n",
        "class Baz\n",
        "  def bar\n",
        "    other\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "end in comment must not corrupt frame stack; got: {:?}",
        diags
    );
}

/// `do` blocks with trailing inline comments must be detected as block openers.
#[test]
fn no_fp_do_block_with_trailing_comment() {
    // `on_load(:x) do # :nodoc:` — the `do` is missed if we only check `ends_with(" do")`.
    // Without the frame push, the `end` closing the block pops the wrong frame.
    let src = concat!(
        "module MyEngine\n",
        "  ActiveSupport.on_load(:action_controller) do # :nodoc:\n",
        "    def helper\n",
        "      true\n",
        "    end\n",
        "  end\n",
        "\n",
        "  def helper\n",
        "    false\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def inside do-block-with-comment must not conflict with outer def; got: {:?}",
        diags
    );
}

/// Three-level nesting: methods in the innermost class must not conflict with
/// methods in middle or outer classes.
#[test]
fn no_fp_three_level_nesting() {
    let src = r#"
module Outer
  class Middle
    class Inner
      def initialize(x)
        @x = x
      end

      def type
        "inner"
      end
    end

    def initialize(y)
      @y = y
    end

    def type
      "middle"
    end
  end

  def type
    "outer"
  end
end
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations in three-level nesting, got: {:?}",
        diags
    );
}

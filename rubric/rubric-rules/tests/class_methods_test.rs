use rubric_core::{LintContext, Rule};
use rubric_rules::style::class_methods::ClassMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/class_methods/offending.rb");

// Basic detection: def ClassName.method_name inside class body must be flagged.
#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClassMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ClassMethods"));
}

// `def self.bar` is correct style — must NOT be flagged.
#[test]
fn no_violation_for_def_self_in_class() {
    let src = "class Foo\n  def self.bar\n    'bar'\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside a class must NOT be flagged, but got: {:?}",
        diags
    );
}

// `def self.bar` inside a module must NOT be flagged — that is correct Ruby style.
// (Style/ModuleFunction is a separate cop.)
#[test]
fn no_violation_for_def_self_in_module() {
    let src = "module Foo\n  def self.bar\n    'bar'\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside a module must NOT be flagged by ClassMethods, but got: {:?}",
        diags
    );
}

// def ClassName.method inside the matching class must be flagged.
#[test]
fn flags_class_name_receiver_in_class() {
    let src = "class MyClass\n  def MyClass.do_thing\n    true\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "def MyClass.method inside class MyClass must be flagged"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/ClassMethods"));
}

// def ModuleName.method inside the matching module must be flagged.
#[test]
fn flags_module_name_receiver_in_module() {
    let src = "module Helpers\n  def Helpers.util\n    42\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "def Helpers.util inside module Helpers must be flagged"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/ClassMethods"));
}

// def OtherName.method — receiver doesn't match enclosing class — must NOT be flagged.
#[test]
fn no_false_positive_for_unrelated_name_receiver() {
    let src = "class Foo\n  def Bar.baz\n    'baz'\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def Bar.baz inside class Foo must NOT be flagged (receiver doesn't match), but got: {:?}",
        diags
    );
}

// Top-level def ClassName.method with no enclosing class — must NOT be flagged.
#[test]
fn no_false_positive_for_top_level_named_def() {
    let src = "def Foo.bar\n  42\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def Foo.bar at top level (no enclosing class) must NOT be flagged, but got: {:?}",
        diags
    );
}

// Nested: def InnerClass.method inside InnerClass nested in a module — must be flagged.
#[test]
fn flags_inner_class_name_receiver_nested_in_module() {
    let src = "module Outer\n  class Inner\n    def Inner.helper\n      true\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "def Inner.helper inside class Inner must be flagged"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/ClassMethods"));
}

// Nested: def OuterModule.method inside InnerClass — must NOT be flagged (receiver is not
// the immediately enclosing class/module).
#[test]
fn no_false_positive_for_outer_name_receiver_inside_nested_class() {
    let src = "module Outer\n  class Inner\n    def Outer.helper\n      true\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def Outer.helper inside class Inner must NOT be flagged (Outer is not the innermost scope), but got: {:?}",
        diags
    );
}

// def self.method at top level (no class/module context) must NOT be flagged.
#[test]
fn no_false_positive_for_top_level_def_self() {
    let src = "def self.top_level\n  42\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method at top level must NOT be flagged, but got: {:?}",
        diags
    );
}

// class << self block is not a class/module scope opener — defs inside it are NOT flagged.
#[test]
fn no_violation_on_class_shovel_self() {
    let src = "class Foo\n  class << self\n    def bar\n      'bar'\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(diags.is_empty(), "class << self block must not produce violations");
}

// class Foo < Bar — class with inheritance — should still detect flagged defs.
#[test]
fn flags_class_name_receiver_with_inheritance() {
    let src = "class Foo < Bar\n  def Foo.build\n    new\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "def Foo.build inside class Foo < Bar must be flagged"
    );
}

// def ClassName.method inside a module that is nested inside a class — must be flagged
// if the receiver matches the innermost module name.
#[test]
fn flags_module_name_receiver_nested_in_class() {
    let src = "class Outer\n  module Utils\n    def Utils.helper\n      true\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "def Utils.helper inside module Utils must be flagged"
    );
}

// Sidekiq pattern: def self.method at top level of a module — NOT flagged by ClassMethods.
// (These were false positives in the old implementation.)
#[test]
fn no_false_positive_for_sidekiq_module_self_methods() {
    let src = concat!(
        "module Sidekiq\n",
        "  def self.server?\n",
        "    defined?(Sidekiq::CLI)\n",
        "  end\n",
        "\n",
        "  def self.redis_pool\n",
        "    Thread.current[:pool]\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside a module must NOT be flagged by ClassMethods, but got: {:?}",
        diags
    );
}

// Devise pattern: def self.method inside a module nested in another module — NOT flagged.
#[test]
fn no_false_positive_for_devise_nested_module_self_methods() {
    let src = concat!(
        "module Devise\n",
        "  module Encryptor\n",
        "    def self.digest(klass, password)\n",
        "      password\n",
        "    end\n",
        "\n",
        "    def self.compare(klass, hashed, password)\n",
        "      true\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside nested modules must NOT be flagged, but got: {:?}",
        diags
    );
}

// Sidekiq tui/tabs pattern: def self.method inside a module nested in a class — NOT flagged.
#[test]
fn no_false_positive_for_module_in_class_def_self_methods() {
    let src = concat!(
        "module Sidekiq\n",
        "  class TUI\n",
        "    module Tabs\n",
        "      def self.all\n",
        "        @all ||= []\n",
        "      end\n",
        "\n",
        "      def self.current\n",
        "        @current ||= all.first\n",
        "      end\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside module nested in class must NOT be flagged, but got: {:?}",
        diags
    );
}

// Mastodon pattern: def self.method at top level of a module — NOT flagged.
#[test]
fn no_false_positive_for_mastodon_trends_module() {
    let src = concat!(
        "module Trends\n",
        "  def self.table_name_prefix\n",
        "    'trends_'\n",
        "  end\n",
        "\n",
        "  def self.links\n",
        "    @links ||= Trends::Links.new\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside a module must NOT be flagged by ClassMethods, but got: {:?}",
        diags
    );
}

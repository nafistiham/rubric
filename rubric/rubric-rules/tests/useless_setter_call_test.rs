use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_setter_call::UselessSetterCall;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/useless_setter_call/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessSetterCall"));
}

#[test]
fn no_violation_on_clean() {
    // self.bar is followed by another statement — not the last line
    let src = "def foo\n  self.bar = 1\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// --- False-positive patterns from mastodon/devise/sidekiq ---

#[test]
fn no_violation_when_rhs_is_method_call() {
    // RHS is a method call (ActiveRecord attribute setter callbacks) — not useless
    let src = "def set_custom_emoji\n  self.custom_emoji = CustomEmoji.local.enabled.find_by(shortcode: name)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "self.foo = method_call() should not be flagged — RHS is not a bare local"
    );
}

#[test]
fn no_violation_when_rhs_is_ternary() {
    // RHS is a ternary expression — not a bare local
    let src = "def filter_not_following=(value)\n  self.for_not_following = value ? :filter : :accept\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "setter method def and ternary RHS should not be flagged"
    );
}

#[test]
fn no_violation_for_setter_method_definition() {
    // The method itself is a setter (`def foo=(value)`) — must write self.attr
    let src = "def tag_name=(new_name)\n  self.tag = Tag.find_or_create_by_names(new_name).first\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "setter method definitions should never be flagged"
    );
}

#[test]
fn no_violation_when_modifier_conditional_present() {
    // self.foo = expr if condition — conditional assignment, not unconditionally useless
    let src = "def set_uri\n  self.uri = ActivityPub::TagManager.instance.generate_uri_for(self) if uri.nil?\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "setter with trailing `if` modifier should not be flagged"
    );
}

#[test]
fn no_violation_when_modifier_unless_present() {
    // self.foo = expr unless condition
    let src = "def prepare_cached_tallies\n  self.cached_tallies = options.map { 0 } unless cached_tallies.empty?\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "setter with trailing `unless` modifier should not be flagged"
    );
}

#[test]
fn no_violation_with_multiple_setters_in_method() {
    // Multiple self.attr = assignments — only the very last line is the potential issue,
    // and even then the RHS must be a bare local for a true positive.
    // Here the RHS is a method call — should not flag.
    let src = concat!(
        "def set_relationships_count!\n",
        "  self.followers_count = severed_relationships.about_local_account(account).passive.count\n",
        "  self.following_count = severed_relationships.about_local_account(account).active.count\n",
        "end\n"
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "multiple setters with method-call RHS should not be flagged"
    );
}

#[test]
fn no_violation_when_setter_method_has_early_return() {
    // Method with `return unless` guard then setter — not useless since it guards a callback
    let src = concat!(
        "def set_published\n",
        "  return unless scheduled_at.blank? || scheduled_at.past?\n",
        "  self.published = true\n",
        "  self.published_at = Time.now.utc\n",
        "end\n"
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "setter callbacks with early return guard should not be flagged"
    );
}

#[test]
fn no_violation_rhs_is_constant() {
    // RHS is a constant — not a bare lowercase local variable
    let src = "def set_type\n  self.type = :unknown\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "setter with symbol literal RHS should not be flagged"
    );
}

#[test]
fn no_violation_rhs_is_integer_literal() {
    // RHS is an integer literal
    let src = "def set_count\n  self.count = 0\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "setter with integer literal RHS should not be flagged"
    );
}

#[test]
fn flags_self_setter_with_bare_local_variable() {
    // The only true positive: last statement is `self.foo = local_var` with no conditional
    let src = "def compute\n  result = calculate()\n  self.value = result\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "self.foo = local_var as last statement should be flagged"
    );
}

#[test]
fn no_violation_when_last_line_inside_begin_end_block() {
    // self.type = begin ... end — complex RHS with embedded begin/end
    let src = concat!(
        "def set_type_and_extension\n",
        "  self.type = begin\n",
        "    if condition\n",
        "      :video\n",
        "    else\n",
        "      :image\n",
        "    end\n",
        "  end\n",
        "end\n"
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "setter with begin..end block RHS should not be flagged"
    );
}

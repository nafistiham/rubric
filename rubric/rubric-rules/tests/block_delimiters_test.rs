use rubric_core::{LintContext, Rule};
use rubric_rules::style::block_delimiters::BlockDelimiters;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/block_delimiters/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/block_delimiters/corrected.rb");

#[test]
fn detects_multiline_brace_block() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/BlockDelimiters"));
}

#[test]
fn no_violation_for_do_end_block() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── Multiline hash literal assigned with `=` must NOT fire ────────────────
// `CONSTANT = {\n  key: value,\n}.freeze` is a hash literal, not a block.
#[test]
fn no_false_positive_for_multiline_hash_literal_assignment() {
    let src = "LEGACY_MAP = {\n  'Foo' => :foo,\n  'Bar' => :bar,\n}.freeze\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "multiline hash literal assignment should not be flagged: {:?}", diags);
}

// ── Hash value inside outer hash must NOT fire ─────────────────────────────
// `mention: {\n  filterable: true,\n}.freeze` — inner hash in hash literal.
#[test]
fn no_false_positive_for_nested_hash_value() {
    let src = "PROPS = {\n  mention: {\n    filterable: true,\n  }.freeze,\n}.freeze\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "nested hash value should not be flagged: {:?}", diags);
}

// ── True positive: multiline brace block after method call still fires ─────
#[test]
fn still_detects_multiline_brace_block() {
    let src = "foo.each {\n  |x|\n  x + 1\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(!diags.is_empty(), "multiline brace block should still be flagged");
}

// ── Lambda body must not be flagged — lambdas require {} ──────────────────
#[test]
fn no_false_positive_for_lambda_body() {
    let src = "handler = -> {\n  puts 'hello'\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "lambda body with -> should not be flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_lambda_with_args() {
    let src = "HANDLER = ->(cli) {\n  cli.stop\n  cli.cleanup\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "lambda with args ->(x) should not be flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_lambda_in_hash() {
    let src = "HANDLERS = {\n  \"TERM\" => ->(cli) {\n    cli.logger.info \"Stopping\"\n    cli.launcher.quiet\n  },\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "lambda in hash value should not be flagged: {:?}", diags);
}

// ── Comment lines with `{` must not be flagged ────────────────────────────
#[test]
fn no_false_positive_for_brace_in_comment() {
    // Lines that are comments (start with `#`) should never be scanned
    let src = "# {\n#   'key' => 'value',\n# }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "brace in a comment line should not be flagged: {:?}", diags);
}

// ── Hash rocket value `'key' => {` must not be flagged ────────────────────
#[test]
fn no_false_positive_for_hash_rocket_value() {
    // `=>` is the hash rocket — the `{` opens a nested hash, not a block
    let src = "config = {\n  'OpenTelemetry' => {\n    use_rack_events: false,\n  },\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "hash rocket value should not be flagged: {:?}", diags);
}

// ── Shovel push of a hash `list << {` must not be flagged ─────────────────
#[test]
fn no_false_positive_for_shovel_push_hash() {
    // `<<` appends a hash to an array — not a block opener
    let src = "entries << {\n  screen_name: name,\n  indices: [0, 5],\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "shovel-pushed hash should not be flagged: {:?}", diags);
}

// ── Percent literal `%r{` on multiple lines must not be flagged ───────────
#[test]
fn no_false_positive_for_percent_regex_literal() {
    // `%r{...}` is a regex literal, not a block
    let src = "REGEX = %r{\n  https?://\n  example\\.com\n}iox\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "percent-regex literal should not be flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_percent_string_literal() {
    // `%{...}` is a percent string literal, not a block
    let src = "banner = %{\n  Welcome\n  to the app\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "percent string literal should not be flagged: {:?}", diags);
}

// ── String interpolation `#{` on multiple lines must not be flagged ───────
#[test]
fn no_false_positive_for_string_interpolation() {
    // `#{` inside a double-quoted string opens an interpolation, not a block
    let src = "\"#<#{self.class.name} #{\n  @options.inspect\n}>\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "string interpolation should not be flagged: {:?}", diags);
}

// ── `lambda {` keyword form must not be flagged ───────────────────────────
#[test]
fn no_false_positive_for_lambda_keyword_form() {
    // `lambda { ... }` is equivalent to `-> { ... }` — not a procedural block
    let src = "scope :active, lambda {\n  where(active: true)\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "lambda keyword form should not be flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_lambda_keyword_with_if() {
    // `lambda {` used as a keyword argument value
    let src = "s.item :x, 'Title', path, if: lambda {\n  current_user.admin?\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "lambda keyword in method args should not be flagged: {:?}", diags);
}

// ── Backslash before `{` (escaped brace in regex body) must not be flagged ──
#[test]
fn no_false_positive_for_escaped_brace_in_regex() {
    // `\{` inside a multi-line regex literal — not a block opener
    let src = "REGEX = /\n  \\{\\{\n    [a-z]+\n  \\}\\}\n/iox\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "escaped brace in regex body should not be flagged: {:?}", diags);
}

// ── Chained block `}.to change {` must not be flagged ────────────────────────
#[test]
fn no_false_positive_for_chained_block_change() {
    // RSpec `change` matcher: `expect { }.to change { }` — braces are correct here
    let src = "expect { subject }.to change {\n  redis.zrange(key, 0, -1)\n}.from([]).to([])\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "chained block after }}.to change should not be flagged: {:?}", diags);
}

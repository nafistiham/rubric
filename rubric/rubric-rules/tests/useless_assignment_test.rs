use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_assignment::UselessAssignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/useless_assignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/useless_assignment/corrected.rb");

#[test]
fn detects_useless_assignment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for unused variable, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessAssignment"));
}

#[test]
fn no_violation_with_all_vars_used() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── False positive: variable assigned with `{...}` block result then used ─────
#[test]
fn no_false_positive_for_curly_block_assignment_then_used() {
    let src = concat!(
        "def queues\n",
        "  Sidekiq.redis do |conn|\n",
        "    queues = conn.sscan('queues').to_a\n",
        "\n",
        "    lengths = conn.pipelined { |pipeline|\n",
        "      queues.each do |queue|\n",
        "        pipeline.llen(queue)\n",
        "      end\n",
        "    }\n",
        "\n",
        "    array_of_arrays = queues.zip(lengths).sort_by { |_, size| -size }\n",
        "    array_of_arrays.to_h\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "curly block assignment falsely flagged: {:?}", diags);
}

// ── False positive: variable assigned via inline `case` then used ─────────────
#[test]
fn no_false_positive_for_inline_case_assignment() {
    let src = "def force_shutdown_after(val)\n  i = case val\n      when :forever\n        -1\n      else\n        Float(val)\n      end\n  @options[:shutdown] = i\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline case assignment falsely flagged: {:?}", diags);
}

// ── False positive: variable used in string interpolation ────────────────────
#[test]
fn no_false_positive_for_string_interpolation_usage() {
    let src = "def build_url(opts)\n  tls_str = opts[:tls] ? '&tls=true' : ''\n  \"ssl://host?#{tls_str}&other\"\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "string interpolation usage falsely flagged: {:?}", diags);
}

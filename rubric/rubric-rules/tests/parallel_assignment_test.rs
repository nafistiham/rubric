use rubric_core::{LintContext, Rule};
use rubric_rules::style::parallel_assignment::ParallelAssignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/parallel_assignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ParallelAssignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "a = 1\nb = 2\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// -- False-positive regression tests --

// RHS is a single method call whose arguments happen to contain commas.
// e.g. `a, b = some_method(arg1, arg2)` -- commas are inside parens, not top-level.
#[test]
fn no_fp_method_call_rhs_with_args() {
    let src = "strategy, delay = delay_for(jobinst, count, exception, msg)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "method call with args on RHS should not be flagged, got: {:?}",
        diags
    );
}

// RHS is a method call with a block that internally has commas.
// e.g. `queue, job = redis { |conn| conn.blocking_call(TIMEOUT, "brpop", *qs, TIMEOUT) }`
#[test]
fn no_fp_method_call_with_block_rhs() {
    let src = "queue, job = redis { |conn| conn.blocking_call(TIMEOUT, \"brpop\", *qs, TIMEOUT) }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "method call with block on RHS should not be flagged, got: {:?}",
        diags
    );
}

// LHS is wrapped in parentheses (multi-assignment destructure from method).
// e.g. `(@current_page, @total_size, @jobs) = page("queue:...", url_params("page"), @count)`
#[test]
fn no_fp_parenthesized_lhs_method_call_rhs() {
    let src = "(@current_page, @total_size, @workset) = page_items(workset, url_params(\"page\"), @count)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "parenthesized LHS with method call RHS should not be flagged, got: {:?}",
        diags
    );
}

// Destructuring from a method returning multiple values -- RHS is a single call.
// e.g. `locale, quality = language.split(";q=", 2)`
#[test]
fn no_fp_split_into_multi_lhs() {
    let src = "locale, quality = language.split(\";q=\", 2)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "destructuring from method call (split) should not be flagged, got: {:?}",
        diags
    );
}

// Another split variant.
// e.g. `score, jid = key.split("-", 2)`
#[test]
fn no_fp_split_two_args() {
    let src = "score, jid = key.split(\"-\", 2)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "split with 2 args should not be flagged, got: {:?}",
        diags
    );
}

// Many LHS variables, single method call RHS.
// e.g. `workers, concurrency, info, rtt = @config.redis { |c| c.hmget(...) }`
#[test]
fn no_fp_many_lhs_single_method_rhs() {
    let src = "workers, concurrency, info, rtt = @config.redis { |c| c.hmget(@id, \"busy\", \"concurrency\", \"info\", \"rtt_us\") }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "many LHS vars with method block RHS should not be flagged, got: {:?}",
        diags
    );
}

// Endless method definition with commas in param list and body.
// e.g. `def head(path, &) = route(:head, path, &)`
#[test]
fn no_fp_endless_method_definition() {
    let src = "def head(path, &) = route(:head, path, &)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "endless method definition should not be flagged, got: {:?}",
        diags
    );
}

// Standard method definition with default args that contain `=` and commas.
// e.g. `def page(key, pageidx = 1, page_size = 25, opts = nil)`
#[test]
fn no_fp_method_def_with_defaults() {
    let src = "def page(key, pageidx = 1, page_size = 25, opts = nil)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "method definition with default args should not be flagged, got: {:?}",
        diags
    );
}

// RHS contains a single bare array variable (no top-level commas).
// e.g. `a, b = some_array`
#[test]
fn no_fp_array_variable_rhs() {
    let src = "a, b = some_array\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "destructuring from array variable should not be flagged, got: {:?}",
        diags
    );
}

// Splat on LHS -- destructuring, not parallel assignment.
// e.g. `first, *rest = some_array`
#[test]
fn no_fp_splat_lhs() {
    let src = "first, *rest = some_array\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "splat LHS should not be flagged, got: {:?}",
        diags
    );
}

// `a, *b = 1, 2, 3` -- splat absorbs extra values; RuboCop does not flag.
#[test]
fn no_fp_splat_rhs_with_literals() {
    let src = "a, *b = 1, 2, 3\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "splat in LHS with literal RHS should not be flagged, got: {:?}",
        diags
    );
}

// RHS top-level comma with an array literal element -- not a plain literal.
// e.g. `buckets, @buckets = @buckets, []`
#[test]
fn no_fp_rhs_contains_array_literal() {
    let src = "buckets, @buckets = @buckets, []\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "RHS containing array literal should not be flagged, got: {:?}",
        diags
    );
}

// `data, score = c.zpopmin(name, 1)&.first` -- method call with safe nav, single RHS value
#[test]
fn no_fp_safe_nav_method_call_rhs() {
    let src = "data, score = c.zpopmin(name, 1)&.first\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "safe-nav method call on RHS should not be flagged, got: {:?}",
        diags
    );
}

// `strat, count = handler.__send__(:delay_for, worker, 2, StandardError.new, {})`
#[test]
fn no_fp_send_method_call_rhs() {
    let src = "strat, count = handler.__send__(:delay_for, worker, 2, StandardError.new, {})\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "__send__ method call RHS should not be flagged, got: {:?}",
        diags
    );
}

// -- True-positive sanity checks --

// `a, b = 1, 2` -- both sides have plain top-level comma-separated literals.
#[test]
fn flags_simple_parallel_assignment() {
    let src = "a, b = 1, 2\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected exactly 1 violation, got: {:?}", diags);
}

// `x, y, z = 'one', 'two', 'three'` -- three string literals on each side.
#[test]
fn flags_three_way_parallel_assignment() {
    let src = "x, y, z = 'one', 'two', 'three'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParallelAssignment.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected exactly 1 violation, got: {:?}", diags);
}

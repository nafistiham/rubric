use rubric_core::{LintContext, Rule};
use rubric_rules::lint::void::Void;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/void/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Void.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/Void"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = 1\ny = x + 1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Implicit return: last expression before `end` in a method body
#[test]
fn no_false_positive_for_implicit_return_before_end() {
    let src = concat!(
        "def weight_mod(sum)\n",
        "  sum == 10 ? 0 : sum\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(diags.is_empty(), "implicit return ternary before end falsely flagged: {:?}", diags);
}

// String literal as format-string operator — implicit return
#[test]
fn no_false_positive_for_format_string_operator() {
    let src = concat!(
        "def uuid\n",
        "  ary = bytes\n",
        "  '%08x-%04x' % ary\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(diags.is_empty(), "format string % operator falsely flagged: {:?}", diags);
}

// Method calls with multiple comma-separated args are NOT void.
// `assert_equal a + b, 10` has a comma so it is a multi-arg call.
// Lines in the middle of a method (NOT last before `end`) must also be skipped.
#[test]
fn no_false_positive_for_method_call_with_arithmetic_arg_in_sequence() {
    // Middle lines don't have `end` on the next line — the implicit-return skip
    // does NOT apply. The comma-skip rule must catch these.
    let src = concat!(
        "def test_sum\n",
        "  assert_equal a + b, 10\n",
        "  assert_equal c + d, 20\n",
        "  assert_equal e + f, 30\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "method call with comma+arithmetic args should not be flagged: {:?}",
        diags
    );
}

// Method call with comparison operator and comma — not void
#[test]
fn no_false_positive_for_assert_with_comparison_and_comma() {
    let src = concat!(
        "def test_comparison\n",
        "  assert_equal x > 0, true\n",
        "  assert_equal y < 10, true\n",
        "  assert_equal z >= 5, true\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "assert with comparison+comma argument should not be flagged: {:?}",
        diags
    );
}

// Branch return value: expression inside if/case branch whose next line is
// `else`, `elsif`, or `when` is the implicit return of that branch — not void.
#[test]
fn no_false_positive_for_branch_return_before_else() {
    let src = concat!(
        "def ratio(a, b)\n",
        "  if a > b\n",
        "    a - b\n",
        "  else\n",
        "    b - a\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(diags.is_empty(), "branch return before `else` falsely flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_branch_return_before_elsif() {
    let src = concat!(
        "def typecast(key, value)\n",
        "  if key == :bool\n",
        "    value == '1'\n",
        "  elsif key == :int\n",
        "    value.to_i\n",
        "  else\n",
        "    value\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(diags.is_empty(), "branch return before `elsif` falsely flagged: {:?}", diags);
}

// `end % n % m` — chained on block result, not standalone void expression
#[test]
fn no_false_positive_for_end_chained_expression() {
    let src = concat!(
        "def inn_checksum(factor, number)\n",
        "  (\n",
        "    factor.map.with_index.reduce(0) do |v, i|\n",
        "      v + i[0] * number[i[1]].to_i\n",
        "    end % 11 % 10\n",
        "  ).to_s\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(diags.is_empty(), "end % n chained expression falsely flagged: {:?}", diags);
}

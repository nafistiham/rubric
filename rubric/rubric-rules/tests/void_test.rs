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

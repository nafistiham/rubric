use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_operation_indentation::MultilineOperationIndentation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/multiline_operation_indentation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/multiline_operation_indentation/corrected.rb");

#[test]
fn detects_bad_multiline_operation_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineOperationIndentation"));
}

#[test]
fn no_violation_for_correct_multiline_operation_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// Regex literal ending with `/` must not be treated as a division operator continuation.
// A line like `unless size =~ /\A[0-9]+x[0-9]+\z/` ends with `/` but the slash closes
// a regex literal — the next line is independent code and must not be flagged.
#[test]
fn no_violation_when_trailing_slash_closes_regex_literal() {
    let src = concat!(
        "      def image(size: '300x300')\n",
        "        raise ArgumentError, 'bad size' unless size =~ /\\A[0-9]+x[0-9]+\\z/\n",
        "        raise ArgumentError, 'bad format' unless format == 'png'\n",
        "      end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for regex-ending line, got: {:?}",
        diags
    );
}

// A regex variable assignment ending with `/` must not trigger a continuation.
// e.g. `@regex = /\A\+(\s|\d)*\z/` followed by `end`
#[test]
fn no_violation_when_regex_assignment_ends_with_slash() {
    let src = concat!(
        "  def setup\n",
        "    @phone_regex = /\\A\\+(\\s|\\d|-|\\(|\\)|x|\\.)*\\z/\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for regex assignment ending with /, got: {:?}",
        diags
    );
}

// Multiple consecutive regex variable assignments (each ending with `/`)
// must not generate cascading false positives.
#[test]
fn no_violation_for_consecutive_regex_assignments() {
    let src = concat!(
        "  def test_patterns\n",
        "    height_pattern = /metre/\n",
        "    length_pattern = /metre/\n",
        "    volume_pattern = /litre|cube/\n",
        "    weight_pattern = /gramme|tonne/\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for consecutive regex assignments, got: {:?}",
        diags
    );
}

// A line ending with `+` inside an already-open parenthesis group should not be
// flagged — the continuation uses alignment indentation, not `current + 2`.
#[test]
fn no_violation_for_plus_continuation_inside_open_paren() {
    let src = concat!(
        "      word_list = (\n",
        "        translate('faker.hipster.words') +\n",
        "        (supplemental ? translate('faker.lorem.words') : [])\n",
        "      )\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for + inside open paren, got: {:?}",
        diags
    );
}

// A chained `&&`/`||` expression spanning 3+ lines must not escalate expected
// indentation. All continuation lines should be at base+2, not previous+2.
//   resource.respond_to?(:a) &&   ← base (6 spaces)
//     resource.respond_to?(:b) &&  ← base+2 = 8 ✓
//     resource.respond_to?(:c)     ← must also be 8, not 10
#[test]
fn no_false_positive_for_chained_continuation_lines() {
    let src = concat!(
        "      resource.respond_to?(:unlock_access!) &&\n",
        "        resource.respond_to?(:unlock_strategy_enabled?) &&\n",
        "        resource.unlock_strategy_enabled?(:email)\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for chained continuation lines, got: {:?}",
        diags
    );
}

// `assert_file "path/to/file", /regex/` contains slashes inside a string literal
// and inside the regex — the string slash must not inflate the slash count so that
// the even-paired guard correctly identifies the trailing `/` as a regex delimiter.
#[test]
fn no_violation_for_regex_arg_with_slashed_path_string() {
    let src = concat!(
        "  test \"route generation\" do\n",
        "    assert_file \"config/routes.rb\", /devise_for :monsters/\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations when path string contains slash, got: {:?}",
        diags
    );
}

// A genuine multiline operation outside any paren group with wrong indentation
// must still be detected.
#[test]
fn detects_wrong_indentation_in_genuine_multiline_operation() {
    let src = "x = foo +\nbar\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for wrong continuation indentation, got none"
    );
    assert_eq!(diags[0].rule, "Layout/MultilineOperationIndentation");
}

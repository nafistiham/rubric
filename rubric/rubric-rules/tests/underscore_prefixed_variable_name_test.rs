use rubric_core::{LintContext, Rule};
use rubric_rules::lint::underscore_prefixed_variable_name::UnderscorePrefixedVariableName;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/underscore_prefixed_variable_name/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnderscorePrefixedVariableName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnderscorePrefixedVariableName"));
}

#[test]
fn no_violation_on_clean() {
    let src = "_foo = 1\n# intentionally unused\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnderscorePrefixedVariableName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Same `_var` assigned (not read) multiple times in separate scopes must NOT fire.
// Each assignment sees the others as "other lines" but they are all LHS assignments,
// not actual reads of the variable.
#[test]
fn no_false_positive_for_repeated_underscore_assignment() {
    let src = "\
it 'a' do
  _tag = create(:tag, name: 'miss')
  results = search('match')
  expect(results).to eq []
end

it 'b' do
  _tag = create(:tag, name: 'miss')
  results = search('other')
  expect(results).to eq []
end
";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnderscorePrefixedVariableName.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`_tag` assigned-only in multiple blocks must not be flagged as used: {:?}",
        diags
    );
}

// A true violation: `_foo` is assigned AND later read.
#[test]
fn still_detects_underscore_var_that_is_read() {
    let src = "_foo = compute\nresult = do_something(_foo)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnderscorePrefixedVariableName.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "`_foo` that is actually read should be flagged"
    );
}

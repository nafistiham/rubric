use rubric_core::{LintContext, Rule};
use rubric_rules::style::hash_syntax::HashSyntax;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/hash_syntax/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/hash_syntax/corrected.rb");

#[test]
fn detects_symbol_rocket_syntax() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = HashSyntax.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for :symbol => rocket syntax, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/HashSyntax"));
}

#[test]
fn no_violation_with_new_hash_syntax() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = HashSyntax.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_namespace_separator_in_rescue() {
    // `::ClassName => var` in rescue clauses must not be flagged as symbol rocket syntax.
    let src = "rescue ActiveRecord::RecordInvalid => e\n  puts e\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashSyntax.check_source(&ctx);
    assert!(diags.is_empty(), "rescue with :: should not be flagged, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_constant_keys_with_namespace() {
    // Hash with constant (non-symbol) keys using `::` must not be flagged.
    let src = "ERRORS = {\n  ActiveRecord::RecordInvalid => 422,\n  HTTP::Error => 503,\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashSyntax.check_source(&ctx);
    assert!(diags.is_empty(), "constant keys with :: should not be flagged, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_mixed_symbol_and_ivar_keys() {
    // A hash with both :symbol => and @ivar => keys must not be flagged.
    // RuboCop's ruby19_no_mixed_keys leaves such hashes alone.
    let src = "{ :out => @stdout, :err => @stderr, @stdin => :close }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashSyntax.check_source(&ctx);
    assert!(diags.is_empty(), "mixed-key hash with ivar keys should not be flagged, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_mixed_symbol_and_string_keys() {
    // Rails params hash mixing :symbol and 'string' rocket keys.
    let src = "post :create, params: { :domain_block => { domain: 'x' }, 'confirm' => '' }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashSyntax.check_source(&ctx);
    assert!(diags.is_empty(), "mixed params hash should not be flagged, got: {:?}", diags);
}

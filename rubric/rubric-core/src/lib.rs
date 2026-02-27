pub mod context;
pub mod rule;
pub mod types;
pub mod walker;

pub use context::LintContext;
pub use rule::Rule;
pub use types::{Diagnostic, Fix, FixSafety, Severity, TextEdit, TextRange};
pub use walker::walk;

#[cfg(test)]
mod prism_api_test {
    #[test]
    fn test_parse_ruby() {
        let result = ruby_prism::parse(b"x = 1\n");
        let _node = result.node();
        // Just verifying the API compiles and runs without panic
        let has_errors = result.errors().next().is_some();
        assert!(!has_errors);
    }
}

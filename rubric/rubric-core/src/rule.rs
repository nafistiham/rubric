use crate::{context::LintContext, types::{Diagnostic, Fix}};

/// Every Rubric cop implements this trait.
///
/// - Source-level rules (line length, trailing whitespace): implement `check_source`.
/// - AST-level rules (string literals, method style): implement `check_node` (added in M2).
pub trait Rule: Send + Sync {
    /// Rubocop-style identifier, e.g. "Layout/TrailingWhitespace".
    fn name(&self) -> &'static str;

    /// Called once per file with the full source.
    /// Override for line-level or whole-file checks.
    fn check_source(&self, _ctx: &LintContext) -> Vec<Diagnostic> {
        vec![]
    }

    /// Produce a fix for the given diagnostic.
    /// Returns `None` if this rule has no auto-fix.
    fn fix(&self, _diag: &Diagnostic) -> Option<Fix> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LintContext;
    use std::path::Path;

    struct NoOpRule;

    impl Rule for NoOpRule {
        fn name(&self) -> &'static str {
            "Test/NoOp"
        }
    }

    #[test]
    fn rule_defaults_return_empty() {
        let source = "x = 1\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        let rule = NoOpRule;
        assert!(rule.check_source(&ctx).is_empty());
        // Create a dummy diagnostic to test fix() default
        use crate::types::{Severity, TextRange};
        let diag = crate::types::Diagnostic {
            rule: "Test/NoOp",
            message: "test".into(),
            range: TextRange::new(0, 1),
            severity: Severity::Warning,
        };
        assert!(rule.fix(&diag).is_none());
    }

    #[test]
    fn rule_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NoOpRule>();
    }
}

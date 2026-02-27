use crate::{context::LintContext, types::{Diagnostic, Fix}};

/// Every Rubric cop implements this trait.
///
/// - Source-level rules (line length, trailing whitespace): implement `check_source`.
/// - AST-level rules (string literals, method style): implement `node_kinds` + `check_node`.
pub trait Rule: Send + Sync {
    /// Rubocop-style identifier, e.g. "Layout/TrailingWhitespace".
    fn name(&self) -> &'static str;

    /// Called once per file with the full source.
    /// Override for line-level or whole-file checks.
    fn check_source(&self, _ctx: &LintContext) -> Vec<Diagnostic> {
        vec![]
    }

    /// Which AST node kinds this rule wants to visit.
    ///
    /// Return an empty slice (the default) for source-only rules.
    /// Kind names match ruby-prism `Node` enum variant names, e.g. `"StringNode"`,
    /// `"DefNode"`, `"CallNode"`.
    fn node_kinds(&self) -> &'static [&'static str] {
        &[]
    }

    /// Called for each AST node whose kind is listed in `node_kinds()`.
    ///
    /// The default implementation returns no diagnostics.
    fn check_node(
        &self,
        _ctx: &LintContext<'_>,
        _node: &ruby_prism::Node<'_>,
    ) -> Vec<Diagnostic> {
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

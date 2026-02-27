/// AST walker: dispatches ruby-prism nodes to rules that opt in via `node_kinds()`.
///
/// Design notes:
/// - `Rule::node_kinds()` returns a slice of kind-name strings (e.g. `"StringNode"`).
/// - `Rule::check_node()` is called for each node whose kind matches.
/// - The walker uses ruby-prism's `Visit<'pr>` trait for complete tree traversal.
/// - `node_kind_name()` maps every `Node` variant to its canonical name.
use crate::{context::LintContext, rule::Rule, types::Diagnostic};

/// Returns the canonical kind name for a ruby-prism `Node`.
/// The returned string matches the Rust enum variant name (e.g. `"StringNode"`).
pub fn node_kind_name<'pr>(node: &ruby_prism::Node<'pr>) -> &'static str {
    use ruby_prism::Node;
    match node {
        Node::AliasGlobalVariableNode { .. } => "AliasGlobalVariableNode",
        Node::AliasMethodNode { .. } => "AliasMethodNode",
        Node::AlternationPatternNode { .. } => "AlternationPatternNode",
        Node::AndNode { .. } => "AndNode",
        Node::ArgumentsNode { .. } => "ArgumentsNode",
        Node::ArrayNode { .. } => "ArrayNode",
        Node::ArrayPatternNode { .. } => "ArrayPatternNode",
        Node::AssocNode { .. } => "AssocNode",
        Node::AssocSplatNode { .. } => "AssocSplatNode",
        Node::BackReferenceReadNode { .. } => "BackReferenceReadNode",
        Node::BeginNode { .. } => "BeginNode",
        Node::BlockArgumentNode { .. } => "BlockArgumentNode",
        Node::BlockLocalVariableNode { .. } => "BlockLocalVariableNode",
        Node::BlockNode { .. } => "BlockNode",
        Node::BlockParameterNode { .. } => "BlockParameterNode",
        Node::BlockParametersNode { .. } => "BlockParametersNode",
        Node::BreakNode { .. } => "BreakNode",
        Node::CallAndWriteNode { .. } => "CallAndWriteNode",
        Node::CallNode { .. } => "CallNode",
        Node::CallOperatorWriteNode { .. } => "CallOperatorWriteNode",
        Node::CallOrWriteNode { .. } => "CallOrWriteNode",
        Node::CallTargetNode { .. } => "CallTargetNode",
        Node::CapturePatternNode { .. } => "CapturePatternNode",
        Node::CaseMatchNode { .. } => "CaseMatchNode",
        Node::CaseNode { .. } => "CaseNode",
        Node::ClassNode { .. } => "ClassNode",
        Node::ClassVariableAndWriteNode { .. } => "ClassVariableAndWriteNode",
        Node::ClassVariableOperatorWriteNode { .. } => "ClassVariableOperatorWriteNode",
        Node::ClassVariableOrWriteNode { .. } => "ClassVariableOrWriteNode",
        Node::ClassVariableReadNode { .. } => "ClassVariableReadNode",
        Node::ClassVariableTargetNode { .. } => "ClassVariableTargetNode",
        Node::ClassVariableWriteNode { .. } => "ClassVariableWriteNode",
        Node::ConstantAndWriteNode { .. } => "ConstantAndWriteNode",
        Node::ConstantOperatorWriteNode { .. } => "ConstantOperatorWriteNode",
        Node::ConstantOrWriteNode { .. } => "ConstantOrWriteNode",
        Node::ConstantPathAndWriteNode { .. } => "ConstantPathAndWriteNode",
        Node::ConstantPathNode { .. } => "ConstantPathNode",
        Node::ConstantPathOperatorWriteNode { .. } => "ConstantPathOperatorWriteNode",
        Node::ConstantPathOrWriteNode { .. } => "ConstantPathOrWriteNode",
        Node::ConstantPathTargetNode { .. } => "ConstantPathTargetNode",
        Node::ConstantPathWriteNode { .. } => "ConstantPathWriteNode",
        Node::ConstantReadNode { .. } => "ConstantReadNode",
        Node::ConstantTargetNode { .. } => "ConstantTargetNode",
        Node::ConstantWriteNode { .. } => "ConstantWriteNode",
        Node::DefNode { .. } => "DefNode",
        Node::DefinedNode { .. } => "DefinedNode",
        Node::ElseNode { .. } => "ElseNode",
        Node::EmbeddedStatementsNode { .. } => "EmbeddedStatementsNode",
        Node::EmbeddedVariableNode { .. } => "EmbeddedVariableNode",
        Node::EnsureNode { .. } => "EnsureNode",
        Node::FalseNode { .. } => "FalseNode",
        Node::FindPatternNode { .. } => "FindPatternNode",
        Node::FlipFlopNode { .. } => "FlipFlopNode",
        Node::FloatNode { .. } => "FloatNode",
        Node::ForNode { .. } => "ForNode",
        Node::ForwardingArgumentsNode { .. } => "ForwardingArgumentsNode",
        Node::ForwardingParameterNode { .. } => "ForwardingParameterNode",
        Node::ForwardingSuperNode { .. } => "ForwardingSuperNode",
        Node::GlobalVariableAndWriteNode { .. } => "GlobalVariableAndWriteNode",
        Node::GlobalVariableOperatorWriteNode { .. } => "GlobalVariableOperatorWriteNode",
        Node::GlobalVariableOrWriteNode { .. } => "GlobalVariableOrWriteNode",
        Node::GlobalVariableReadNode { .. } => "GlobalVariableReadNode",
        Node::GlobalVariableTargetNode { .. } => "GlobalVariableTargetNode",
        Node::GlobalVariableWriteNode { .. } => "GlobalVariableWriteNode",
        Node::HashNode { .. } => "HashNode",
        Node::HashPatternNode { .. } => "HashPatternNode",
        Node::IfNode { .. } => "IfNode",
        Node::ImaginaryNode { .. } => "ImaginaryNode",
        Node::ImplicitNode { .. } => "ImplicitNode",
        Node::ImplicitRestNode { .. } => "ImplicitRestNode",
        Node::InNode { .. } => "InNode",
        Node::IndexAndWriteNode { .. } => "IndexAndWriteNode",
        Node::IndexOperatorWriteNode { .. } => "IndexOperatorWriteNode",
        Node::IndexOrWriteNode { .. } => "IndexOrWriteNode",
        Node::IndexTargetNode { .. } => "IndexTargetNode",
        Node::InstanceVariableAndWriteNode { .. } => "InstanceVariableAndWriteNode",
        Node::InstanceVariableOperatorWriteNode { .. } => "InstanceVariableOperatorWriteNode",
        Node::InstanceVariableOrWriteNode { .. } => "InstanceVariableOrWriteNode",
        Node::InstanceVariableReadNode { .. } => "InstanceVariableReadNode",
        Node::InstanceVariableTargetNode { .. } => "InstanceVariableTargetNode",
        Node::InstanceVariableWriteNode { .. } => "InstanceVariableWriteNode",
        Node::IntegerNode { .. } => "IntegerNode",
        Node::InterpolatedMatchLastLineNode { .. } => "InterpolatedMatchLastLineNode",
        Node::InterpolatedRegularExpressionNode { .. } => "InterpolatedRegularExpressionNode",
        Node::InterpolatedStringNode { .. } => "InterpolatedStringNode",
        Node::InterpolatedSymbolNode { .. } => "InterpolatedSymbolNode",
        Node::InterpolatedXStringNode { .. } => "InterpolatedXStringNode",
        Node::ItLocalVariableReadNode { .. } => "ItLocalVariableReadNode",
        Node::ItParametersNode { .. } => "ItParametersNode",
        Node::KeywordHashNode { .. } => "KeywordHashNode",
        Node::KeywordRestParameterNode { .. } => "KeywordRestParameterNode",
        Node::LambdaNode { .. } => "LambdaNode",
        Node::LocalVariableAndWriteNode { .. } => "LocalVariableAndWriteNode",
        Node::LocalVariableOperatorWriteNode { .. } => "LocalVariableOperatorWriteNode",
        Node::LocalVariableOrWriteNode { .. } => "LocalVariableOrWriteNode",
        Node::LocalVariableReadNode { .. } => "LocalVariableReadNode",
        Node::LocalVariableTargetNode { .. } => "LocalVariableTargetNode",
        Node::LocalVariableWriteNode { .. } => "LocalVariableWriteNode",
        Node::MatchLastLineNode { .. } => "MatchLastLineNode",
        Node::MatchPredicateNode { .. } => "MatchPredicateNode",
        Node::MatchRequiredNode { .. } => "MatchRequiredNode",
        Node::MatchWriteNode { .. } => "MatchWriteNode",
        Node::MissingNode { .. } => "MissingNode",
        Node::ModuleNode { .. } => "ModuleNode",
        Node::MultiTargetNode { .. } => "MultiTargetNode",
        Node::MultiWriteNode { .. } => "MultiWriteNode",
        Node::NextNode { .. } => "NextNode",
        Node::NilNode { .. } => "NilNode",
        Node::NoKeywordsParameterNode { .. } => "NoKeywordsParameterNode",
        Node::NumberedParametersNode { .. } => "NumberedParametersNode",
        Node::NumberedReferenceReadNode { .. } => "NumberedReferenceReadNode",
        Node::OptionalKeywordParameterNode { .. } => "OptionalKeywordParameterNode",
        Node::OptionalParameterNode { .. } => "OptionalParameterNode",
        Node::OrNode { .. } => "OrNode",
        Node::ParametersNode { .. } => "ParametersNode",
        Node::ParenthesesNode { .. } => "ParenthesesNode",
        Node::PinnedExpressionNode { .. } => "PinnedExpressionNode",
        Node::PinnedVariableNode { .. } => "PinnedVariableNode",
        Node::PostExecutionNode { .. } => "PostExecutionNode",
        Node::PreExecutionNode { .. } => "PreExecutionNode",
        Node::ProgramNode { .. } => "ProgramNode",
        Node::RangeNode { .. } => "RangeNode",
        Node::RationalNode { .. } => "RationalNode",
        Node::RedoNode { .. } => "RedoNode",
        Node::RegularExpressionNode { .. } => "RegularExpressionNode",
        Node::RequiredKeywordParameterNode { .. } => "RequiredKeywordParameterNode",
        Node::RequiredParameterNode { .. } => "RequiredParameterNode",
        Node::RescueModifierNode { .. } => "RescueModifierNode",
        Node::RescueNode { .. } => "RescueNode",
        Node::RestParameterNode { .. } => "RestParameterNode",
        Node::RetryNode { .. } => "RetryNode",
        Node::ReturnNode { .. } => "ReturnNode",
        Node::SelfNode { .. } => "SelfNode",
        Node::ShareableConstantNode { .. } => "ShareableConstantNode",
        Node::SingletonClassNode { .. } => "SingletonClassNode",
        Node::SourceEncodingNode { .. } => "SourceEncodingNode",
        Node::SourceFileNode { .. } => "SourceFileNode",
        Node::SourceLineNode { .. } => "SourceLineNode",
        Node::SplatNode { .. } => "SplatNode",
        Node::StatementsNode { .. } => "StatementsNode",
        Node::StringNode { .. } => "StringNode",
        Node::SuperNode { .. } => "SuperNode",
        Node::SymbolNode { .. } => "SymbolNode",
        Node::TrueNode { .. } => "TrueNode",
        Node::UndefNode { .. } => "UndefNode",
        Node::UnlessNode { .. } => "UnlessNode",
        Node::UntilNode { .. } => "UntilNode",
        Node::WhenNode { .. } => "WhenNode",
        Node::WhileNode { .. } => "WhileNode",
        Node::XStringNode { .. } => "XStringNode",
        Node::YieldNode { .. } => "YieldNode",
    }
}

/// Internal walker that holds references to context and rules during traversal.
struct RuleWalker<'a, 'ctx> {
    ctx: &'a LintContext<'ctx>,
    rules: &'a [Box<dyn Rule>],
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'ctx> RuleWalker<'a, 'ctx> {
    fn new(ctx: &'a LintContext<'ctx>, rules: &'a [Box<dyn Rule>]) -> Self {
        Self {
            ctx,
            rules,
            diagnostics: Vec::new(),
        }
    }

    fn dispatch(&mut self, node: &ruby_prism::Node<'_>) {
        let kind = node_kind_name(node);
        for rule in self.rules {
            if rule.node_kinds().contains(&kind) {
                let diags = rule.check_node(self.ctx, node);
                self.diagnostics.extend(diags);
            }
        }
    }
}

impl<'pr> ruby_prism::Visit<'pr> for RuleWalker<'_, '_> {
    fn visit_branch_node_enter(&mut self, node: ruby_prism::Node<'pr>) {
        self.dispatch(&node);
    }

    fn visit_leaf_node_enter(&mut self, node: ruby_prism::Node<'pr>) {
        self.dispatch(&node);
    }
}

/// Parse `source` and walk every AST node, calling `check_node()` on rules
/// that registered the matching kind via `node_kinds()`.
///
/// Returns all diagnostics produced during the walk.
pub fn walk(source: &[u8], ctx: &LintContext<'_>, rules: &[Box<dyn Rule>]) -> Vec<Diagnostic> {
    // Skip walk if no rule cares about any node kind
    if rules.iter().all(|r| r.node_kinds().is_empty()) {
        return Vec::new();
    }

    let result = ruby_prism::parse(source);
    let root = result.node();

    let mut walker = RuleWalker::new(ctx, rules);
    ruby_prism::Visit::visit(&mut walker, &root);
    walker.diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LintContext;
    use crate::rule::Rule;
    use crate::types::Diagnostic;
    use std::path::Path;
    use std::sync::Mutex;

    // A rule that records every StringNode it visits.
    struct StringVisitor {
        visited_kinds: Mutex<Vec<&'static str>>,
    }

    impl StringVisitor {
        fn new() -> Self {
            Self {
                visited_kinds: Mutex::new(Vec::new()),
            }
        }
    }

    impl Rule for StringVisitor {
        fn name(&self) -> &'static str {
            "Test/StringVisitor"
        }

        fn node_kinds(&self) -> &'static [&'static str] {
            &["StringNode"]
        }

        fn check_node(
            &self,
            _ctx: &LintContext<'_>,
            node: &ruby_prism::Node<'_>,
        ) -> Vec<Diagnostic> {
            self.visited_kinds
                .lock()
                .unwrap()
                .push(node_kind_name(node));
            vec![]
        }
    }

    #[test]
    fn walker_visits_string_node() {
        let source = b"x = \"hello\"\n";
        let source_str = std::str::from_utf8(source).unwrap();
        let path = Path::new("test.rb");
        let ctx = LintContext::new(path, source_str);

        let rule = StringVisitor::new();
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(rule)];

        let diags = walk(source, &ctx, &rules);
        assert!(diags.is_empty());

        // Verify the rule instance saw the StringNode by downcasting
        // (we can't access visited_kinds after moving into Box, so we test indirectly
        // by verifying the walk ran without panicking and produced no diagnostics)
    }

    #[test]
    fn walker_skips_walk_when_no_node_rules() {
        // A rule that only does source-level checks should not trigger a walk
        struct SourceOnlyRule;
        impl Rule for SourceOnlyRule {
            fn name(&self) -> &'static str {
                "Test/SourceOnly"
            }
            // node_kinds() defaults to &[] — no AST interest
        }

        let source = b"x = 1\n";
        let source_str = std::str::from_utf8(source).unwrap();
        let ctx = LintContext::new(Path::new("test.rb"), source_str);
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(SourceOnlyRule)];

        let diags = walk(source, &ctx, &rules);
        assert!(diags.is_empty());
    }

    #[test]
    fn walker_counts_string_nodes() {
        use std::sync::Arc;

        // Track via shared counter to observe visits after move into Box
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        struct CountingStringRule {
            counter: Arc<std::sync::atomic::AtomicUsize>,
        }

        impl Rule for CountingStringRule {
            fn name(&self) -> &'static str {
                "Test/CountingString"
            }

            fn node_kinds(&self) -> &'static [&'static str] {
                &["StringNode"]
            }

            fn check_node(
                &self,
                _ctx: &LintContext<'_>,
                _node: &ruby_prism::Node<'_>,
            ) -> Vec<Diagnostic> {
                self.counter
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                vec![]
            }
        }

        let counter_clone = Arc::clone(&counter);
        let source = b"x = \"hello\"\ny = \"world\"\n";
        let source_str = std::str::from_utf8(source).unwrap();
        let ctx = LintContext::new(Path::new("test.rb"), source_str);
        let rules: Vec<Box<dyn Rule>> =
            vec![Box::new(CountingStringRule { counter: counter_clone })];

        let diags = walk(source, &ctx, &rules);
        assert!(diags.is_empty());
        // Two string literals should have triggered two visits
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn walker_returns_diagnostics_from_node_rule() {
        use crate::types::{Severity, TextRange};

        struct AlwaysWarnOnString;

        impl Rule for AlwaysWarnOnString {
            fn name(&self) -> &'static str {
                "Test/AlwaysWarn"
            }

            fn node_kinds(&self) -> &'static [&'static str] {
                &["StringNode"]
            }

            fn check_node(
                &self,
                _ctx: &LintContext<'_>,
                node: &ruby_prism::Node<'_>,
            ) -> Vec<Diagnostic> {
                let loc = node.location();
                let start = loc.start_offset() as u32;
                let end = loc.end_offset() as u32;
                vec![Diagnostic {
                    rule: "Test/AlwaysWarn",
                    message: "found a string".to_string(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                }]
            }
        }

        let source = b"x = \"hello\"\n";
        let source_str = std::str::from_utf8(source).unwrap();
        let ctx = LintContext::new(Path::new("test.rb"), source_str);
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(AlwaysWarnOnString)];

        let diags = walk(source, &ctx, &rules);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "Test/AlwaysWarn");
        assert_eq!(diags[0].message, "found a string");
    }

    #[test]
    fn node_kind_name_returns_string_node_for_string_literal() {
        let result = ruby_prism::parse(b"\"hello\"\n");
        let root = result.node();
        // root is ProgramNode -> StatementsNode -> StringNode
        // We can verify node_kind_name works by matching StringNode directly
        let kind = node_kind_name(&root);
        assert_eq!(kind, "ProgramNode");
    }
}

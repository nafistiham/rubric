use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct StringLiterals;

impl Rule for StringLiterals {
    fn name(&self) -> &'static str {
        "Style/StringLiterals"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["StringNode"]
    }

    fn check_node(
        &self,
        ctx: &LintContext<'_>,
        node: &ruby_prism::Node<'_>,
    ) -> Vec<Diagnostic> {
        let loc = node.location();
        let start = loc.start_offset();
        let end = loc.end_offset();
        let src_bytes = ctx.source.as_bytes();
        if start >= src_bytes.len() || end > src_bytes.len() {
            return vec![];
        }
        let node_src = &src_bytes[start..end];

        // Only flag double-quoted strings (not heredocs, %q{}, etc.)
        if node_src.first() != Some(&b'"') {
            return vec![];
        }

        // Content between the quotes
        if node_src.len() < 2 {
            return vec![];
        }
        let content = &node_src[1..node_src.len() - 1];

        // Guard against string segments inside interpolation (child nodes of InterpolatedStringNode)
        if content.windows(2).any(|w| w == b"#{") {
            return vec![];
        }

        // Contains single quote -> can't convert
        if content.contains(&b'\'') {
            return vec![];
        }
        // Contains backslash -> has escape sequences, don't convert
        if content.contains(&b'\\') {
            return vec![];
        }

        vec![Diagnostic {
            rule: self.name(),
            message: "Prefer single-quoted strings when interpolation is not needed.".into(),
            range: TextRange::new(start as u32, end as u32),
            severity: Severity::Warning,
        }]
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

/// `Style/StringLiterals` — enforces consistent string quoting style.
///
/// `double_quotes: false` (default) → prefer single-quoted strings.
/// `double_quotes: true`  → prefer double-quoted strings.
pub struct StringLiterals {
    pub double_quotes: bool,
}

impl Default for StringLiterals {
    fn default() -> Self {
        Self { double_quotes: false }
    }
}

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
        if node_src.len() < 2 {
            return vec![];
        }

        // Skip inner string fragments (parts of an interpolated `%(...)` or similar) that
        // have no opening delimiter. These fragments don't start with a quote char as delimiter.
        let has_opening_delim = node.as_string_node()
            .and_then(|n| n.opening_loc())
            .is_some();
        if !has_opening_delim {
            return vec![];
        }

        if self.double_quotes {
            // Prefer double-quoted: flag single-quoted strings
            if node_src.first() != Some(&b'\'') {
                return vec![];
            }
            let content = &node_src[1..node_src.len() - 1];
            // Skip if it contains a double-quote (would need escaping)
            if content.contains(&b'"') {
                return vec![];
            }
            vec![Diagnostic {
                rule: self.name(),
                message: "Prefer double-quoted strings unless the string contains double quotes.".into(),
                range: TextRange::new(start as u32, end as u32),
                severity: Severity::Warning,
            }]
        } else {
            // Prefer single-quoted: flag double-quoted strings (default)
            if node_src.first() != Some(&b'"') {
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
}

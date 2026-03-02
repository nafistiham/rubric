use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashMap;

pub struct DuplicateHashKey;

/// Scan `hash_bytes` (the raw source bytes of a single hash literal) for
/// duplicate keys, returning diagnostics for any duplicates found.
///
/// - `abs_start`: byte offset of `hash_bytes[0]` in the overall source.
/// - `direct_depth`: the bracket-nesting depth at which direct keys appear.
///   - 1 for `HashNode`        (source starts with `{`)
///   - 0 for `KeywordHashNode` (implicit hash, no surrounding braces)
/// - `rule_name`: the rule identifier string for diagnostic tagging.
fn scan_hash_for_duplicates(
    hash_bytes: &[u8],
    abs_start: usize,
    direct_depth: usize,
    rule_name: &'static str,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut seen: HashMap<String, ()> = HashMap::new();
    let mut i = 0;
    let n = hash_bytes.len();
    let mut depth: usize = 0;
    let mut in_string: Option<u8> = None;

    while i < n {
        let b = hash_bytes[i];

        // 1. String state: skip content until closing delimiter (handles escapes).
        if let Some(delim) = in_string {
            if b == b'\\' { i += 2; continue; }
            if b == delim { in_string = None; }
            i += 1;
            continue;
        }

        // 2. Depth tracking: ALL bracket types increase/decrease nesting.
        match b {
            b'{' | b'[' | b'(' => { depth += 1; i += 1; continue; }
            b'}' | b']' | b')' => {
                if depth > 0 { depth -= 1; }
                i += 1;
                continue;
            }
            _ => {}
        }

        // 3. Skip characters not at the direct depth (but start string tracking
        //    so that nested strings don't confuse the key scanner).
        if depth != direct_depth {
            if b == b'"' || b == b'\'' {
                in_string = Some(b);
            }
            i += 1;
            continue;
        }

        // 4. Skip inline comments at direct depth.
        if b == b'#' {
            while i < n && hash_bytes[i] != b'\n' { i += 1; }
            continue;
        }

        // 5. Pattern 1: `word:` new-style symbol key (e.g. `name: value`).
        if b.is_ascii_alphabetic() || b == b'_' {
            let ki = i;
            while i < n && (hash_bytes[i].is_ascii_alphanumeric() || hash_bytes[i] == b'_') {
                i += 1;
            }
            // Must be followed by `:` but NOT `::` (namespace separator).
            if i < n && hash_bytes[i] == b':' && (i + 1 >= n || hash_bytes[i + 1] != b':') {
                let key_text = std::str::from_utf8(&hash_bytes[ki..i]).unwrap_or("?");
                let canonical = format!("sym:{}", key_text);
                let abs_pos = (abs_start + ki) as u32;
                if seen.contains_key(&canonical) {
                    diags.push(Diagnostic {
                        rule: rule_name,
                        message: format!("Duplicate hash key `{}`.", key_text),
                        range: TextRange::new(abs_pos, abs_pos + (i - ki) as u32),
                        severity: Severity::Warning,
                    });
                } else {
                    seen.insert(canonical, ());
                }
            }
            continue;
        }

        // 6. Pattern 2: `'str' =>` or `"str" =>` old-style string key.
        if b == b'\'' || b == b'"' {
            let delim = b;
            i += 1; // consume opening quote
            let ki = i;
            while i < n {
                if hash_bytes[i] == b'\\' { i += 2; continue; }
                if hash_bytes[i] == delim { break; }
                i += 1;
            }
            let key_text = std::str::from_utf8(&hash_bytes[ki..i]).unwrap_or("?").to_string();
            let key_abs = (abs_start + ki - 1) as u32; // include opening quote
            if i < n { i += 1; } // consume closing quote
            // Skip optional whitespace/newlines.
            while i < n && matches!(hash_bytes[i], b' ' | b'\t' | b'\n' | b'\r') { i += 1; }
            // Only treat as a key if followed by `=>`.
            if i + 1 < n && hash_bytes[i] == b'=' && hash_bytes[i + 1] == b'>' {
                let canonical = format!("str:{}", key_text);
                if seen.contains_key(&canonical) {
                    let range_end = key_abs + key_text.len() as u32 + 2; // +2 for quotes
                    diags.push(Diagnostic {
                        rule: rule_name,
                        message: format!("Duplicate hash key `'{}'`.", key_text),
                        range: TextRange::new(key_abs, range_end),
                        severity: Severity::Warning,
                    });
                } else {
                    seen.insert(canonical, ());
                }
            }
            continue;
        }

        i += 1;
    }

    diags
}

impl Rule for DuplicateHashKey {
    fn name(&self) -> &'static str {
        "Lint/DuplicateHashKey"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["HashNode", "KeywordHashNode"]
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
        let hash_bytes = &src_bytes[start..end];

        // HashNode source starts with `{`, so direct keys are at depth 1.
        // KeywordHashNode has no surrounding braces, so direct keys are at depth 0.
        let direct_depth = match node {
            ruby_prism::Node::HashNode { .. } => 1,
            ruby_prism::Node::KeywordHashNode { .. } => 0,
            _ => return vec![],
        };

        scan_hash_for_duplicates(hash_bytes, start, direct_depth, self.name())
    }
}

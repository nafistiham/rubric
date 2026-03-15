use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HashTransformKeys;

impl Rule for HashTransformKeys {
    fn name(&self) -> &'static str {
        "Style/HashTransformKeys"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;

            // Detect `.map { |k, v| [k.` — key is being transformed (method call on k),
            // value is presumably unchanged. Combined with `.to_h` → suggest transform_keys.
            if line.contains(".to_h") {
                if let Some(map_pos) = find_outside_string(line, ".map") {
                    let after_map = &line[map_pos + ".map".len()..];
                    // Must have `|k, v|` or `|k,v|` block param pattern
                    if contains_pattern(after_map, "|k, v|") || contains_pattern(after_map, "|k,v|") {
                        // Key is transformed: look for `[k.` (key gets a method call)
                        // and absence of a value transform (value stays as `v]` or `, v]`).
                        if contains_pattern(after_map, "[k.") {
                            let start = (line_start + map_pos) as u32;
                            let end = (line_start + map_pos + ".map".len()) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Use transform_keys instead of map.to_h.".to_string(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        diags
    }
}

/// Returns the byte position of `needle` in `haystack`, skipping string literals and comments.
fn find_outside_string(haystack: &str, needle: &str) -> Option<usize> {
    let bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();
    let n = bytes.len();
    let m = needle_bytes.len();
    let mut in_string: Option<u8> = None;
    let mut i = 0;

    while i < n {
        let b = bytes[i];

        if let Some(delim) = in_string {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == delim {
                in_string = None;
            }
            i += 1;
            continue;
        }

        match b {
            b'"' | b'\'' | b'`' => {
                in_string = Some(b);
            }
            b'#' => break, // comment — stop scanning
            _ => {
                if i + m <= n && &bytes[i..i + m] == needle_bytes {
                    return Some(i);
                }
            }
        }
        i += 1;
    }
    None
}

/// Returns true if `pattern` appears literally in `s`.
fn contains_pattern(s: &str, pattern: &str) -> bool {
    s.contains(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn check(source: &str) -> Vec<Diagnostic> {
        let ctx = LintContext::new(Path::new("test.rb"), source);
        HashTransformKeys.check_source(&ctx)
    }

    #[test]
    fn flags_map_with_key_transform_to_h() {
        let diags = check("hash.map { |k, v| [k.to_s, v] }.to_h\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "Style/HashTransformKeys");
        assert!(diags[0].message.contains("transform_keys"));
    }

    #[test]
    fn flags_map_key_transform_with_symbol_method() {
        let diags = check("opts.map { |k, v| [k.to_sym, v] }.to_h\n");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn no_flag_when_value_is_transformed_not_key() {
        // Key kept as `k` (no method call), value is transformed → transform_values territory
        let diags = check("hash.map { |k, v| [k, v.to_s] }.to_h\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_map_without_to_h() {
        let diags = check("hash.map { |k, v| [k.to_s, v] }\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_inside_comment() {
        let diags = check("# hash.map { |k, v| [k.to_s, v] }.to_h\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_inside_string() {
        let diags = check("x = \"hash.map { |k, v| [k.to_s, v] }.to_h\"\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_plain_map() {
        let diags = check("arr.map { |x| x.upcase }\n");
        assert_eq!(diags.len(), 0);
    }
}

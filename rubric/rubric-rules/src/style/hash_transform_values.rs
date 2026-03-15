use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HashTransformValues;

impl Rule for HashTransformValues {
    fn name(&self) -> &'static str {
        "Style/HashTransformValues"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;

            // Detect `.each_with_object({})` with block param `|(k, v)` — strongest signal
            if let Some(pos) = find_outside_string(line, ".each_with_object({})") {
                // Confirm the block param pattern indicates hash key/value decomposition
                let after = &line[pos + ".each_with_object({})".len()..];
                let after_trimmed = after.trim_start();
                if after_trimmed.starts_with("{ |(k, v)")
                    || after_trimmed.starts_with("{|(k, v)")
                    || after_trimmed.starts_with("do |(k, v)")
                {
                    let start = (line_start + pos) as u32;
                    let end = (line_start + pos + ".each_with_object({})".len()) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use transform_values instead of each_with_object or map.to_h."
                            .to_string(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                    continue;
                }
            }

            // Detect `.map { |k, v| [k,` — key unchanged, value transformed, then `.to_h`
            // Look for `.map` with `|k, v|` block and `[k,` inside (key kept, value transformed)
            if line.contains(".to_h") {
                if let Some(map_pos) = find_outside_string(line, ".map") {
                    let after_map = &line[map_pos + ".map".len()..];
                    // Must have `|k, v|` block param pattern
                    if contains_pattern(after_map, "|k, v|") || contains_pattern(after_map, "|k,v|") {
                        // Must have `[k,` inside (key unchanged) to distinguish from transform_keys
                        if contains_pattern(after_map, "[k,") || contains_pattern(after_map, "[k ,") {
                            let start = (line_start + map_pos) as u32;
                            let end = (line_start + map_pos + ".map".len()) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message:
                                    "Use transform_values instead of each_with_object or map.to_h."
                                        .to_string(),
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

/// Returns true if `pattern` appears literally in `s` (simple substring check, no string tracking).
fn contains_pattern(s: &str, pattern: &str) -> bool {
    s.contains(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn check(source: &str) -> Vec<Diagnostic> {
        let ctx = LintContext::new(Path::new("test.rb"), source);
        HashTransformValues.check_source(&ctx)
    }

    // --- each_with_object({}) pattern ---

    #[test]
    fn flags_each_with_object_with_kv_block_param() {
        let diags = check("hash.each_with_object({}) { |(k, v), h| h[k] = v.to_s }\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "Style/HashTransformValues");
        assert!(diags[0].message.contains("transform_values"));
    }

    #[test]
    fn flags_each_with_object_do_block_variant() {
        let diags = check("hash.each_with_object({}) do |(k, v), h|\n");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn no_flag_each_with_object_without_kv_param() {
        // e.g. accumulating into array, not key/value hash
        let diags = check("arr.each_with_object([]) { |x, acc| acc << x }\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_inside_comment() {
        let diags = check("# hash.each_with_object({}) { |(k, v), h| h[k] = v.to_s }\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_inside_string() {
        let diags = check("x = \"hash.each_with_object({}) { |(k, v), h| h[k] = v.to_s }\"\n");
        assert_eq!(diags.len(), 0);
    }

    // --- map { |k, v| [k, ...] }.to_h pattern ---

    #[test]
    fn flags_map_with_kv_block_and_key_preserved_to_h() {
        let diags = check("hash.map { |k, v| [k, v.to_s] }.to_h\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("transform_values"));
    }

    #[test]
    fn no_flag_map_to_h_without_k_preserved() {
        // key is transformed — this is transform_keys territory, not transform_values
        let diags = check("hash.map { |k, v| [k.to_s, v] }.to_h\n");
        // key gets method call, `[k,` not present literally → no flag for transform_values
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_map_without_to_h() {
        let diags = check("hash.map { |k, v| [k, v.to_s] }\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn no_flag_plain_map_array() {
        let diags = check("arr.map { |x| x * 2 }\n");
        assert_eq!(diags.len(), 0);
    }
}

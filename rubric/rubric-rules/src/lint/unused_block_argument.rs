use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UnusedBlockArgument;

/// True if `name` appears as a word-boundary token anywhere in `source`.
fn name_used_in_source(name: &[u8], source: &[u8]) -> bool {
    let n = source.len();
    let vn = name.len();
    if vn == 0 || n < vn {
        return false;
    }
    let mut pos = 0;
    while pos + vn <= n {
        if &source[pos..pos + vn] == name {
            let before_ok = pos == 0 || {
                let b = source[pos - 1];
                !b.is_ascii_alphanumeric() && b != b'_'
            };
            let after_ok = pos + vn >= n || {
                let b = source[pos + vn];
                !b.is_ascii_alphanumeric() && b != b'_'
            };
            if before_ok && after_ok {
                return true;
            }
        }
        pos += 1;
    }
    false
}

impl Rule for UnusedBlockArgument {
    fn name(&self) -> &'static str {
        "Lint/UnusedBlockArgument"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["BlockNode"]
    }

    fn check_node(&self, ctx: &LintContext<'_>, node: &ruby_prism::Node<'_>) -> Vec<Diagnostic> {
        let block_node = match node.as_block_node() {
            Some(b) => b,
            None => return vec![],
        };

        // No body → empty block, nothing to flag
        let body = match block_node.body() {
            Some(b) => b,
            None => return vec![],
        };

        // No parameters at all
        let params_node = match block_node.parameters() {
            Some(p) => p,
            None => return vec![],
        };

        // BlockNode.parameters() can be a BlockParametersNode (|a, b|) or
        // NumberedParametersNode (_1, _2) or ItParametersNode (it).
        // Only BlockParametersNode has named params we can check.
        let block_params = match params_node.as_block_parameters_node() {
            Some(bp) => bp,
            None => return vec![],
        };

        let inner_params = match block_params.parameters() {
            Some(p) => p,
            None => return vec![],
        };

        // Collect required positional params (the common case: |x, y|)
        let mut param_list: Vec<(Vec<u8>, u32, u32)> = Vec::new();

        for node in inner_params.requireds().iter() {
            if let Some(p) = node.as_required_parameter_node() {
                let loc = p.location();
                param_list.push((p.name().as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
            }
            // RequiredDestructuredParameterNode (|(a, b)|) — too complex, skip
        }

        for node in inner_params.optionals().iter() {
            if let Some(p) = node.as_optional_parameter_node() {
                let loc = p.name_loc();
                param_list.push((p.name().as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
            }
        }

        if let Some(rest) = inner_params.rest() {
            if let Some(p) = rest.as_rest_parameter_node() {
                if let (Some(name), Some(loc)) = (p.name(), p.name_loc()) {
                    param_list.push((name.as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
                }
            }
        }

        if param_list.is_empty() {
            return vec![];
        }

        // Body source range
        let body_loc = body.location();
        let body_start = body_loc.start_offset();
        let body_end = body_loc.end_offset();
        let src = ctx.source.as_bytes();
        if body_end > src.len() {
            return vec![];
        }
        let body_src = &src[body_start..body_end];

        let mut diags = Vec::new();
        for (name_bytes, name_start, name_end) in &param_list {
            // _-prefixed: intentionally unused
            if name_bytes.first() == Some(&b'_') {
                continue;
            }
            if name_bytes.is_empty() {
                continue;
            }
            if !name_used_in_source(name_bytes, body_src) {
                let name_str = std::str::from_utf8(name_bytes).unwrap_or("?");
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "Block argument `{}` is unused; prefix with `_` to suppress.",
                        name_str
                    ),
                    range: TextRange::new(*name_start, *name_end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

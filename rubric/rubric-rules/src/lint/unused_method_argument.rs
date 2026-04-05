use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UnusedMethodArgument;

/// Collect (name_bytes, name_start, name_end) for all named parameters in a
/// `ParametersNode`.  Anonymous params (`*`, `**`, `&`) produce no entry.
fn collect_method_params(params: &ruby_prism::ParametersNode<'_>) -> Vec<(Vec<u8>, u32, u32)> {
    let mut result: Vec<(Vec<u8>, u32, u32)> = Vec::new();

    // Required positional: def foo(a, b)
    for node in params.requireds().iter() {
        if let Some(p) = node.as_required_parameter_node() {
            let loc = p.location();
            result.push((p.name().as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
        }
        // RequiredDestructuredParameterNode (def foo((a, b))) — too complex, skip
    }

    // Optional positional: def foo(a = 1)
    for node in params.optionals().iter() {
        if let Some(p) = node.as_optional_parameter_node() {
            let loc = p.name_loc();
            result.push((p.name().as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
        }
    }

    // Rest: def foo(*args)  — only if named
    if let Some(rest) = params.rest() {
        if let Some(p) = rest.as_rest_parameter_node() {
            if let (Some(name), Some(loc)) = (p.name(), p.name_loc()) {
                result.push((name.as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
            }
        }
    }

    // Post-rest required: def foo(*args, b)
    for node in params.posts().iter() {
        if let Some(p) = node.as_required_parameter_node() {
            let loc = p.location();
            result.push((p.name().as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
        }
    }

    // Keyword required: def foo(name:)
    for node in params.keywords().iter() {
        if let Some(p) = node.as_required_keyword_parameter_node() {
            let loc = p.name_loc();
            result.push((p.name().as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
        } else if let Some(p) = node.as_optional_keyword_parameter_node() {
            let loc = p.name_loc();
            result.push((p.name().as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
        }
    }

    // Keyword rest: def foo(**opts)  — only if named
    if let Some(kw_rest) = params.keyword_rest() {
        if let Some(p) = kw_rest.as_keyword_rest_parameter_node() {
            if let (Some(name), Some(loc)) = (p.name(), p.name_loc()) {
                result.push((name.as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
            }
        }
        // NoKeywordsParameterNode (**nil) and ForwardingParameterNode (...) are
        // silently skipped — no name to flag.
    }

    // Block: def foo(&block)  — only if named
    if let Some(bp) = params.block() {
        if let (Some(name), Some(loc)) = (bp.name(), bp.name_loc()) {
            result.push((name.as_slice().to_vec(), loc.start_offset() as u32, loc.end_offset() as u32));
        }
    }

    result
}

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

impl Rule for UnusedMethodArgument {
    fn name(&self) -> &'static str {
        "Lint/UnusedMethodArgument"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["DefNode"]
    }

    fn check_node(&self, ctx: &LintContext<'_>, node: &ruby_prism::Node<'_>) -> Vec<Diagnostic> {
        let def_node = match node.as_def_node() {
            Some(d) => d,
            None => return vec![],
        };

        // Endless method: def foo(n) = n * 2  — has no separate body node
        if def_node.equal_loc().is_some() {
            return vec![];
        }

        // No body (pure forward-declaration stub / empty method)
        let body = match def_node.body() {
            Some(b) => b,
            None => return vec![],
        };

        // No parameters at all
        let params = match def_node.parameters() {
            Some(p) => p,
            None => return vec![],
        };

        // If any required param is a ForwardingParameterNode (...), skip entirely —
        // all arguments are forwarded and nothing should be flagged.
        let has_forwarding = params.requireds().iter()
            .any(|n| n.as_forwarding_parameter_node().is_some());
        if has_forwarding {
            return vec![];
        }

        let param_list = collect_method_params(&params);
        if param_list.is_empty() {
            return vec![];
        }

        // Body source range.
        // NOTE: In prism, heredoc nodes have location() pointing only to the
        // opening `<<~TERM` token, not the body lines. The heredoc body bytes
        // are physically present in the file between the opening statement line
        // and the `end` keyword. To capture them we extend the search range to
        // the end of the entire `def...end` block.
        let body_start = body.location().start_offset();
        let def_end = def_node.location().end_offset();
        let src = ctx.source.as_bytes();
        if def_end > src.len() {
            return vec![];
        }
        let body_src = &src[body_start..def_end];

        // IgnoreNotImplementedMethods: skip methods whose body is just
        // `raise NotImplementedError` / `fail NotImplementedError` (any variant).
        let body_trimmed = std::str::from_utf8(body_src).unwrap_or("").trim();
        let is_not_implemented = {
            let b = body_trimmed;
            b.starts_with("raise NotImplementedError") || b.starts_with("fail NotImplementedError")
                || b.starts_with("raise AbstractMethodNotOverriddenError")
                || b.starts_with("fail AbstractMethodNotOverriddenError")
        };
        if is_not_implemented {
            return vec![];
        }

        // If the body contains bare `super` (not `super()` which passes nothing),
        // all arguments are implicitly forwarded — nothing to flag.
        if name_used_in_source(b"super", body_src) {
            // Check it's not `super()` — bare super passes all args, super() passes none.
            // We scan for `super` not immediately followed by `(`.
            let mut has_bare_super = false;
            let mut pos = 0;
            while pos + 5 <= body_src.len() {
                if &body_src[pos..pos + 5] == b"super" {
                    let before_ok = pos == 0 || {
                        let b = body_src[pos - 1];
                        !b.is_ascii_alphanumeric() && b != b'_'
                    };
                    let after = if pos + 5 < body_src.len() { body_src[pos + 5] } else { b'\n' };
                    let after_ok = !after.is_ascii_alphanumeric() && after != b'_';
                    if before_ok && after_ok && after != b'(' {
                        has_bare_super = true;
                        break;
                    }
                }
                pos += 1;
            }
            if has_bare_super {
                return vec![];
            }
        }

        let mut diags = Vec::new();
        for (name_bytes, name_start, name_end) in &param_list {
            // _-prefixed: intentionally unused
            if name_bytes.first() == Some(&b'_') {
                continue;
            }
            if name_bytes.is_empty() {
                continue;
            }
            // Search from the end of this parameter's name to the end of the
            // entire `def...end` block. This covers:
            //   1. The rest of the parameter list (e.g. `withvalues: false,
            //      with_values: withvalues` — `withvalues` used as default)
            //   2. The method body (including heredoc bodies via def_end)
            let search_start = (*name_end as usize).min(def_end);
            let search_src = &src[search_start..def_end];
            if !name_used_in_source(name_bytes, search_src) {
                let name_str = std::str::from_utf8(name_bytes).unwrap_or("?");
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!("Unused method argument `{}`.", name_str),
                    range: TextRange::new(*name_start, *name_end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

use rubric_core::{Diagnostic, LintContext, Rule};

pub struct TrailingBodyOnMethodDefinition;

impl Rule for TrailingBodyOnMethodDefinition {
    fn name(&self) -> &'static str {
        "Style/TrailingBodyOnMethodDefinition"
    }

    fn check_source(&self, _ctx: &LintContext) -> Vec<Diagnostic> {
        vec![]
    }
}

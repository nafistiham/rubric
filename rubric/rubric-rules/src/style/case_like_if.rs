use rubric_core::{Diagnostic, LintContext, Rule};

pub struct CaseLikeIf;

impl Rule for CaseLikeIf {
    fn name(&self) -> &'static str {
        "Style/CaseLikeIf"
    }

    fn check_source(&self, _ctx: &LintContext) -> Vec<Diagnostic> {
        vec![]
    }
}

use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceAfterMethodName;

impl Rule for SpaceAfterMethodName {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterMethodName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Only look at lines starting with `def `
            if !trimmed.starts_with("def ") {
                continue;
            }
            // Find pattern: def <word> (
            // Look for word chars followed by a space and then '('
            let after_def = &trimmed[4..]; // skip "def "
            let bytes = after_def.as_bytes();
            let mut j = 0;
            // Skip the method name (word characters and ? ! =)
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'?' || bytes[j] == b'!' || bytes[j] == b'=') {
                j += 1;
            }
            // Now check if there's a space followed by '('
            if j < bytes.len() && bytes[j] == b' ' {
                let mut k = j + 1;
                // Skip additional spaces
                while k < bytes.len() && bytes[k] == b' ' {
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b'(' {
                    // Violation: space between method name and '('
                    let indent_len = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    // Point to the space character
                    let space_start = line_start + indent_len + 4 + j;
                    let space_end = line_start + indent_len + 4 + k;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Space between method name and `(` in definition.".into(),
                        range: TextRange::new(space_start as u32, space_end as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

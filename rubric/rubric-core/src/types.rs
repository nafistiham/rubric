/// A byte-offset range within a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

impl TextRange {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A lint violation reported by a rule.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub rule: &'static str, // e.g. "Layout/TrailingWhitespace"
    pub message: String,
    pub range: TextRange,
    pub severity: Severity,
}

/// A single text substitution that resolves a violation.
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: TextRange,
    pub replacement: String,
}

/// Whether applying this fix could change program behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixSafety {
    Safe,
    Unsafe,
}

/// A complete auto-fix for a diagnostic (may contain multiple edits).
#[derive(Debug, Clone)]
pub struct Fix {
    pub edits: Vec<TextEdit>,
    pub safety: FixSafety,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_range_new_stores_start_and_end() {
        let r = TextRange::new(0, 10);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 10);
    }

    #[test]
    fn text_range_equality() {
        assert_eq!(TextRange::new(1, 5), TextRange::new(1, 5));
        assert_ne!(TextRange::new(1, 5), TextRange::new(1, 6));
    }

    #[test]
    fn text_range_is_copy() {
        let r = TextRange::new(3, 7);
        let r2 = r; // Copy — r is still usable
        assert_eq!(r, r2);
    }

    #[test]
    fn severity_variants_are_distinct() {
        assert_ne!(Severity::Error, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Info);
        assert_ne!(Severity::Error, Severity::Info);
    }

    #[test]
    fn diagnostic_fields_are_accessible() {
        let d = Diagnostic {
            rule: "Layout/TrailingWhitespace",
            message: "trailing whitespace".to_string(),
            range: TextRange::new(0, 5),
            severity: Severity::Warning,
        };
        assert_eq!(d.rule, "Layout/TrailingWhitespace");
        assert_eq!(d.message, "trailing whitespace");
        assert_eq!(d.range, TextRange::new(0, 5));
        assert_eq!(d.severity, Severity::Warning);
    }

    #[test]
    fn diagnostic_is_cloneable() {
        let d = Diagnostic {
            rule: "Style/StringLiterals",
            message: "prefer single quotes".to_string(),
            range: TextRange::new(2, 8),
            severity: Severity::Info,
        };
        let d2 = d.clone();
        assert_eq!(d2.rule, d.rule);
        assert_eq!(d2.message, d.message);
    }

    #[test]
    fn text_edit_fields_are_accessible() {
        let e = TextEdit {
            range: TextRange::new(10, 15),
            replacement: "foo".to_string(),
        };
        assert_eq!(e.range, TextRange::new(10, 15));
        assert_eq!(e.replacement, "foo");
    }

    #[test]
    fn fix_safety_variants_are_distinct() {
        assert_ne!(FixSafety::Safe, FixSafety::Unsafe);
    }

    #[test]
    fn fix_holds_multiple_edits() {
        let fix = Fix {
            edits: vec![
                TextEdit { range: TextRange::new(0, 3), replacement: "abc".to_string() },
                TextEdit { range: TextRange::new(5, 8), replacement: "xyz".to_string() },
            ],
            safety: FixSafety::Safe,
        };
        assert_eq!(fix.edits.len(), 2);
        assert_eq!(fix.safety, FixSafety::Safe);
    }

    #[test]
    fn fix_is_cloneable() {
        let fix = Fix {
            edits: vec![TextEdit {
                range: TextRange::new(0, 1),
                replacement: "x".to_string(),
            }],
            safety: FixSafety::Unsafe,
        };
        let fix2 = fix.clone();
        assert_eq!(fix2.edits.len(), 1);
        assert_eq!(fix2.safety, FixSafety::Unsafe);
    }
}

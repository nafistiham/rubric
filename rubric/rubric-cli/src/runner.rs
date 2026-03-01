use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use rayon::prelude::*;
use rubric_core::{LintContext, Rule};

pub fn collect_ruby_files(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().and_then(|s| s.to_str()) == Some("rb")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Run all rules against the given context, returning all diagnostics.
/// Runs both source-level checks and AST-level checks (via walker).
#[allow(dead_code)]
pub fn run_rules_on_source(
    ctx: &LintContext,
    rules: &[Box<dyn Rule>],
) -> Vec<rubric_core::Diagnostic> {
    rules.iter().flat_map(|rule| rule.check_source(ctx)).collect()
}

/// Process multiple files in parallel using Rayon.
/// Returns (path, source, diagnostics) triples, order is non-deterministic.
pub fn run_all_files(
    files: &[PathBuf],
    rules: &[Box<dyn Rule + Send + Sync>],
) -> Vec<(PathBuf, String, Vec<rubric_core::Diagnostic>)> {
    files
        .par_iter()
        .filter_map(|path| {
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: could not read {}: {e}", path.display());
                    return None;
                }
            };
            let ctx = LintContext::new(path, &source);

            // Source-level checks
            let mut diagnostics: Vec<rubric_core::Diagnostic> =
                rules.iter().flat_map(|rule| rule.check_source(&ctx)).collect();

            // AST-level checks — only if any rule registers node_kinds
            if rules.iter().any(|r| !r.node_kinds().is_empty()) {
                let ast_diags = rubric_core::walk(source.as_bytes(), &ctx, rules);
                diagnostics.extend(ast_diags);
            }

            Some((path.clone(), source, diagnostics))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
    use std::fs;
    use tempfile::TempDir;

    struct AlwaysWarn;

    impl Rule for AlwaysWarn {
        fn name(&self) -> &'static str {
            "Test/AlwaysWarn"
        }

        fn check_source(&self, _ctx: &LintContext) -> Vec<Diagnostic> {
            vec![Diagnostic {
                rule: self.name(),
                message: "always warns".into(),
                range: TextRange::new(0, 1),
                severity: Severity::Warning,
            }]
        }
    }

    struct NeverWarn;

    impl Rule for NeverWarn {
        fn name(&self) -> &'static str {
            "Test/NeverWarn"
        }
    }

    fn make_temp_dir() -> TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    // collect_ruby_files tests

    #[test]
    fn collect_ruby_files_returns_rb_files() {
        let dir = make_temp_dir();
        fs::write(dir.path().join("foo.rb"), "").unwrap();
        fs::write(dir.path().join("bar.rb"), "").unwrap();

        let mut files = collect_ruby_files(dir.path());
        files.sort();

        assert_eq!(files.len(), 2);
        assert!(files[0].extension().unwrap() == "rb");
        assert!(files[1].extension().unwrap() == "rb");
    }

    #[test]
    fn collect_ruby_files_ignores_non_rb_files() {
        let dir = make_temp_dir();
        fs::write(dir.path().join("foo.rb"), "").unwrap();
        fs::write(dir.path().join("bar.py"), "").unwrap();
        fs::write(dir.path().join("baz.txt"), "").unwrap();

        let files = collect_ruby_files(dir.path());

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("foo.rb"));
    }

    #[test]
    fn collect_ruby_files_returns_empty_for_no_rb_files() {
        let dir = make_temp_dir();
        fs::write(dir.path().join("main.py"), "").unwrap();

        let files = collect_ruby_files(dir.path());

        assert!(files.is_empty());
    }

    #[test]
    fn collect_ruby_files_walks_subdirectories() {
        let dir = make_temp_dir();
        let sub = dir.path().join("lib");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("nested.rb"), "").unwrap();
        fs::write(dir.path().join("top.rb"), "").unwrap();

        let files = collect_ruby_files(dir.path());

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn collect_ruby_files_does_not_include_directories() {
        let dir = make_temp_dir();
        let sub = dir.path().join("my_dir.rb"); // directory with .rb name
        fs::create_dir(&sub).unwrap();
        fs::write(dir.path().join("real.rb"), "").unwrap();

        let files = collect_ruby_files(dir.path());

        // Only the real file should be included, not the directory
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("real.rb"));
    }

    #[test]
    fn collect_ruby_files_accepts_single_file_path() {
        let dir = make_temp_dir();
        let file = dir.path().join("single.rb");
        fs::write(&file, "").unwrap();

        let files = collect_ruby_files(&file);

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("single.rb"));
    }

    // run_rules_on_source tests

    #[test]
    fn run_rules_on_source_returns_diagnostics_from_rules() {
        let dir = make_temp_dir();
        let file = dir.path().join("test.rb");
        fs::write(&file, "x = 1\n").unwrap();

        let source = fs::read_to_string(&file).unwrap();
        let ctx = LintContext::new(&file, &source);
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(AlwaysWarn)];
        let diagnostics = run_rules_on_source(&ctx, &rules);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "Test/AlwaysWarn");
    }

    #[test]
    fn run_rules_on_source_returns_empty_when_no_violations() {
        let dir = make_temp_dir();
        let file = dir.path().join("clean.rb");
        fs::write(&file, "x = 1\n").unwrap();

        let source = fs::read_to_string(&file).unwrap();
        let ctx = LintContext::new(&file, &source);
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(NeverWarn)];
        let diagnostics = run_rules_on_source(&ctx, &rules);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn run_rules_on_source_collects_from_multiple_rules() {
        let dir = make_temp_dir();
        let file = dir.path().join("multi.rb");
        fs::write(&file, "x = 1\n").unwrap();

        let source = fs::read_to_string(&file).unwrap();
        let ctx = LintContext::new(&file, &source);
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(AlwaysWarn), Box::new(AlwaysWarn)];
        let diagnostics = run_rules_on_source(&ctx, &rules);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn run_rules_returns_empty_for_empty_source() {
        use rubric_rules::TrailingWhitespace;
        let ctx = LintContext::new(Path::new("test.rb"), "");
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(TrailingWhitespace)];
        let diagnostics = run_rules_on_source(&ctx, &rules);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn run_all_files_processes_multiple_files_in_parallel() {
        use rubric_rules::TrailingWhitespace;
        let dir = tempfile::tempdir().expect("temp dir");
        // File with trailing whitespace
        let f1 = dir.path().join("a.rb");
        std::fs::write(&f1, "def foo   \nend\n").unwrap();
        // Clean file
        let f2 = dir.path().join("b.rb");
        std::fs::write(&f2, "def bar\nend\n").unwrap();

        let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![Box::new(TrailingWhitespace)];
        let files = vec![f1.clone(), f2.clone()];
        let mut results = run_all_files(&files, &rules);
        results.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, f1);
        assert_eq!(results[0].2.len(), 1); // 1 violation in a.rb
        assert_eq!(results[1].0, f2);
        assert_eq!(results[1].2.len(), 0); // 0 violations in b.rb
    }

    #[test]
    fn run_all_files_returns_source_string() {
        use rubric_rules::TrailingWhitespace;
        let dir = tempfile::tempdir().expect("temp dir");
        let f1 = dir.path().join("src_test.rb");
        std::fs::write(&f1, "x = 1\n").unwrap();

        let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![Box::new(TrailingWhitespace)];
        let files = vec![f1.clone()];
        let results = run_all_files(&files, &rules);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, f1);
        assert_eq!(results[0].1, "x = 1\n"); // source is returned
        assert_eq!(results[0].2.len(), 0);
    }

    #[test]
    fn run_all_files_runs_ast_level_rules() {
        use rubric_rules::StringLiterals;
        let dir = tempfile::tempdir().expect("temp dir");
        let f1 = dir.path().join("strings.rb");
        // Double-quoted strings without interpolation should be flagged
        std::fs::write(&f1, "x = \"hello\"\n").unwrap();

        let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![Box::new(StringLiterals)];
        let files = vec![f1.clone()];
        let results = run_all_files(&files, &rules);

        assert_eq!(results.len(), 1);
        // StringLiterals uses AST walker — should detect double-quoted string
        assert_eq!(results[0].2.len(), 1);
        assert_eq!(results[0].2[0].rule, "Style/StringLiterals");
    }
}

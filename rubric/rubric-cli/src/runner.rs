use std::path::Path;
use anyhow::Result;
use walkdir::WalkDir;
use rubric_core::{LintContext, Rule};

pub fn collect_ruby_files(path: &Path) -> Vec<std::path::PathBuf> {
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

#[allow(dead_code)]
pub fn run_rules_on_file(
    path: &Path,
    rules: &[Box<dyn Rule>],
) -> Result<Vec<rubric_core::Diagnostic>> {
    let source = std::fs::read_to_string(path)?;
    let ctx = LintContext::new(path, &source);
    let mut diagnostics = Vec::new();
    for rule in rules {
        diagnostics.extend(rule.check_source(&ctx));
    }
    Ok(diagnostics)
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

    // run_rules_on_file tests

    #[test]
    fn run_rules_on_file_returns_diagnostics_from_rules() {
        let dir = make_temp_dir();
        let file = dir.path().join("test.rb");
        fs::write(&file, "x = 1\n").unwrap();

        let rules: Vec<Box<dyn Rule>> = vec![Box::new(AlwaysWarn)];
        let diagnostics = run_rules_on_file(&file, &rules).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "Test/AlwaysWarn");
    }

    #[test]
    fn run_rules_on_file_returns_empty_when_no_violations() {
        let dir = make_temp_dir();
        let file = dir.path().join("clean.rb");
        fs::write(&file, "x = 1\n").unwrap();

        let rules: Vec<Box<dyn Rule>> = vec![Box::new(NeverWarn)];
        let diagnostics = run_rules_on_file(&file, &rules).unwrap();

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn run_rules_on_file_collects_from_multiple_rules() {
        let dir = make_temp_dir();
        let file = dir.path().join("multi.rb");
        fs::write(&file, "x = 1\n").unwrap();

        let rules: Vec<Box<dyn Rule>> = vec![Box::new(AlwaysWarn), Box::new(AlwaysWarn)];
        let diagnostics = run_rules_on_file(&file, &rules).unwrap();

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn run_rules_on_file_errors_on_missing_file() {
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(NeverWarn)];
        let result = run_rules_on_file(Path::new("/nonexistent/path.rb"), &rules);

        assert!(result.is_err());
    }
}

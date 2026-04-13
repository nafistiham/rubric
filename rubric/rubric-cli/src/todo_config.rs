//! `.rubric_todo.toml` loading — suppresses known (cop, file) violations during check.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RawTodoEntry {
    #[serde(default)]
    files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawTodoFile {
    #[serde(default)]
    todo: HashMap<String, RawTodoEntry>,
}

/// The parsed contents of `.rubric_todo.toml`.
/// Maps cop name → set of relative file paths that are suppressed.
#[derive(Debug, Default)]
pub struct TodoConfig {
    /// cop_name → set of relative file paths (as stored in the file, e.g. "lib/foo.rb")
    suppressed: HashMap<String, HashSet<String>>,
}

impl TodoConfig {
    /// Load `.rubric_todo.toml` from `dir`. Returns an empty config if the file doesn't exist.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(".rubric_todo.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        if content.trim().is_empty() {
            return Ok(Self::default());
        }
        let raw: RawTodoFile = toml::from_str(&content)
            .with_context(|| format!("parsing {}", path.display()))?;

        let suppressed = raw.todo
            .into_iter()
            .map(|(cop, entry)| (cop, entry.files.into_iter().collect()))
            .collect();

        Ok(Self { suppressed })
    }

    /// Returns true if this (cop, relative_file_path) pair is suppressed.
    pub fn is_suppressed(&self, cop: &str, rel_path: &str) -> bool {
        self.suppressed
            .get(cop)
            .map_or(false, |files| files.contains(rel_path))
    }

    /// Returns true if the todo file has any suppressions at all.
    pub fn is_empty(&self) -> bool {
        self.suppressed.is_empty()
    }

    /// Total number of (cop, file) suppression pairs.
    #[allow(dead_code)]
    pub fn total_suppressions(&self) -> usize {
        self.suppressed.values().map(|s| s.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_returns_empty_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = TodoConfig::load(dir.path()).unwrap();
        assert!(cfg.is_empty());
        assert!(!cfg.is_suppressed("Style/Documentation", "lib/foo.rb"));
    }

    #[test]
    fn load_parses_single_cop_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut f = std::fs::File::create(dir.path().join(".rubric_todo.toml")).unwrap();
        writeln!(f, "[todo.\"Style/Documentation\"]\nfiles = [\"lib/foo.rb\"]").unwrap();

        let cfg = TodoConfig::load(dir.path()).unwrap();
        assert!(cfg.is_suppressed("Style/Documentation", "lib/foo.rb"));
        assert!(!cfg.is_suppressed("Style/Documentation", "lib/bar.rb"));
        assert!(!cfg.is_suppressed("Layout/LineLength", "lib/foo.rb"));
    }

    #[test]
    fn load_parses_multiple_cops() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".rubric_todo.toml"),
            r#"
[todo."Style/Documentation"]
files = ["lib/a.rb", "lib/b.rb"]

[todo."Layout/LineLength"]
files = ["lib/a.rb"]
"#,
        )
        .unwrap();

        let cfg = TodoConfig::load(dir.path()).unwrap();
        assert!(cfg.is_suppressed("Style/Documentation", "lib/a.rb"));
        assert!(cfg.is_suppressed("Style/Documentation", "lib/b.rb"));
        assert!(cfg.is_suppressed("Layout/LineLength", "lib/a.rb"));
        assert!(!cfg.is_suppressed("Layout/LineLength", "lib/b.rb"));
        assert_eq!(cfg.total_suppressions(), 3);
    }

    #[test]
    fn load_returns_empty_for_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".rubric_todo.toml"), "").unwrap();
        let cfg = TodoConfig::load(dir.path()).unwrap();
        assert!(cfg.is_empty());
    }
}

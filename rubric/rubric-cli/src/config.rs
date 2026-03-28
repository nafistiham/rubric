//! `rubric.toml` configuration loading.
//! Fields are read by rule-filtering and formatter dispatch logic in M3.
#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Per-cop config parsed from e.g. `[rules."Style/Foo"] enabled = false`.
#[derive(Debug, Deserialize, Default)]
struct RawRuleConfig {
    #[serde(default = "default_true")]
    enabled: bool,
    /// Glob patterns for files to exclude from this rule (relative to project root).
    #[serde(default)]
    exclude: Vec<String>,
    /// Maximum line length (for Layout/LineLength).
    max: Option<u64>,
    /// Style enforcement string (e.g. "double_quotes", "no_space").
    enforced_style: Option<String>,
}

/// Intermediate struct that mirrors the raw TOML layout.
/// `exclude` may appear either at the top level or under `[formatter]`.
#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    linter: RawLinterConfig,
    #[serde(default)]
    formatter: RawFormatterConfig,
    #[serde(default)]
    exclude: Vec<String>,
    /// Per-cop enable/disable table. Keys are cop names like "Style/StringLiterals".
    #[serde(default)]
    rules: HashMap<String, RawRuleConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct RawLinterConfig {
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    disabled_by_default: bool,
}

#[derive(Debug, Deserialize, Default)]
struct RawFormatterConfig {
    #[serde(default = "default_true")]
    enabled: bool,
    /// `exclude` may be nested under `[formatter]` in the TOML file.
    #[serde(default)]
    exclude: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Public per-cop config.
#[derive(Debug)]
pub struct RuleConfig {
    pub enabled: bool,
    /// Glob patterns for files to exclude from this rule (relative to project root).
    pub exclude: Vec<String>,
    /// Maximum line length override (for Layout/LineLength).
    pub max: Option<u64>,
    /// Style enforcement string (e.g. "double_quotes", "no_space").
    pub enforced_style: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub linter: LinterConfig,
    pub formatter: FormatterConfig,
    /// Exclude glob patterns — merged from top-level and `[formatter].exclude`.
    pub exclude: Vec<String>,
    /// Per-cop enable/disable map. Keys are cop names like "Style/StringLiterals".
    pub rules: HashMap<String, RuleConfig>,
}

#[derive(Debug)]
pub struct LinterConfig {
    pub enabled: bool,
    pub disabled_by_default: bool,
}

#[derive(Debug)]
pub struct FormatterConfig {
    pub enabled: bool,
}

impl Config {
    /// Returns true if the named rule should run.
    /// When `disabled_by_default` is set, unlisted rules return false.
    pub fn is_rule_enabled(&self, name: &str) -> bool {
        match self.rules.get(name) {
            Some(r) => r.enabled,
            None => !self.linter.disabled_by_default,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            linter: LinterConfig { enabled: true, disabled_by_default: false },
            formatter: FormatterConfig { enabled: true },
            exclude: Vec::new(),
            rules: HashMap::new(),
        }
    }
}

impl Config {
    /// Load `rubric.toml` from `dir`. Returns default config if the file doesn't exist.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join("rubric.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        if content.trim().is_empty() {
            return Ok(Self::default());
        }
        let raw: RawConfig = toml::from_str(&content)
            .with_context(|| format!("parsing {}", path.display()))?;

        // Merge both exclude lists; deduplicate while preserving order.
        let mut exclude = raw.exclude;
        for pat in raw.formatter.exclude {
            if !exclude.contains(&pat) {
                exclude.push(pat);
            }
        }

        let rules = raw.rules.into_iter()
            .map(|(k, v)| (k, RuleConfig { enabled: v.enabled, exclude: v.exclude, max: v.max, enforced_style: v.enforced_style }))
            .collect();

        Ok(Self {
            linter: LinterConfig {
                enabled: raw.linter.enabled,
                disabled_by_default: raw.linter.disabled_by_default,
            },
            formatter: FormatterConfig {
                enabled: raw.formatter.enabled,
            },
            exclude,
            rules,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn config_load_returns_defaults_when_no_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let config = Config::load(dir.path()).expect("load");
        assert_eq!(config.linter.enabled, true);
        assert_eq!(config.formatter.enabled, true);
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn config_load_parses_rubric_toml() {
        let dir = tempfile::tempdir().expect("temp dir");
        let toml_path = dir.path().join("rubric.toml");
        let mut f = std::fs::File::create(&toml_path).unwrap();
        writeln!(f, "[linter]\nenabled = false\n[formatter]\nenabled = true\nexclude = [\"vendor/**\"]").unwrap();

        let config = Config::load(dir.path()).expect("load");
        assert_eq!(config.linter.enabled, false);
        assert_eq!(config.formatter.enabled, true);
        assert_eq!(config.exclude, vec!["vendor/**"]);
    }

    #[test]
    fn config_load_returns_defaults_for_empty_toml() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("rubric.toml"), "").unwrap();
        let config = Config::load(dir.path()).expect("load");
        assert_eq!(config.linter.enabled, true);
        assert_eq!(config.formatter.enabled, true);
    }

    #[test]
    fn config_load_parses_per_cop_rules() {
        let dir = tempfile::tempdir().expect("temp dir");
        let toml_path = dir.path().join("rubric.toml");
        std::fs::write(
            &toml_path,
            r#"
[rules."Style/StringLiterals"]
enabled = false

[rules."Layout/LineLength"]
enabled = false
"#,
        )
        .unwrap();

        let config = Config::load(dir.path()).expect("load");
        assert!(!config.is_rule_enabled("Style/StringLiterals"));
        assert!(!config.is_rule_enabled("Layout/LineLength"));
    }

    #[test]
    fn config_unknown_rule_defaults_to_enabled() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("rubric.toml"), "").unwrap();
        let config = Config::load(dir.path()).expect("load");
        assert!(config.is_rule_enabled("Style/UnknownCop"));
    }

    #[test]
    fn config_rule_enabled_true_is_accepted() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(
            dir.path().join("rubric.toml"),
            "[rules.\"Style/TrailingCommaInArguments\"]\nenabled = true\n",
        )
        .unwrap();
        let config = Config::load(dir.path()).expect("load");
        assert!(config.is_rule_enabled("Style/TrailingCommaInArguments"));
    }

    #[test]
    fn config_disabled_by_default_skips_unlisted_rules() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(
            dir.path().join("rubric.toml"),
            "[linter]\ndisabled_by_default = true\n\n[rules.\"Layout/TrailingWhitespace\"]\nenabled = true\n",
        ).unwrap();
        let config = Config::load(dir.path()).expect("load");
        assert!(config.is_rule_enabled("Layout/TrailingWhitespace"));
        assert!(!config.is_rule_enabled("Style/StringLiterals")); // not listed → disabled
    }
}

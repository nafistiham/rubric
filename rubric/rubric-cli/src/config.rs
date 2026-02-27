//! `rubric.toml` configuration loading.
//! Fields are read by rule-filtering and formatter dispatch logic in M3.
#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

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
}

#[derive(Debug, Deserialize, Default)]
struct RawLinterConfig {
    #[serde(default = "default_true")]
    enabled: bool,
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

#[derive(Debug)]
pub struct Config {
    pub linter: LinterConfig,
    pub formatter: FormatterConfig,
    /// Exclude glob patterns — merged from top-level and `[formatter].exclude`.
    pub exclude: Vec<String>,
}

#[derive(Debug)]
pub struct LinterConfig {
    // Will be read by rule-filtering logic in M3.
    #[allow(dead_code)]
    pub enabled: bool,
}

#[derive(Debug)]
pub struct FormatterConfig {
    // Will be read by formatter dispatch logic in M3.
    #[allow(dead_code)]
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            linter: LinterConfig { enabled: true },
            formatter: FormatterConfig { enabled: true },
            exclude: Vec::new(),
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

        // Merge exclude: top-level takes precedence; fall back to formatter.exclude.
        let exclude = if !raw.exclude.is_empty() {
            raw.exclude
        } else {
            raw.formatter.exclude
        };

        Ok(Self {
            linter: LinterConfig {
                enabled: raw.linter.enabled,
            },
            formatter: FormatterConfig {
                enabled: raw.formatter.enabled,
            },
            exclude,
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
}

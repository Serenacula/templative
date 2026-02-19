use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::errors::TemplativeError;
use crate::utilities;

const CONFIG_VERSION: u32 = 1;
const CONFIG_FILENAME: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum GitMode {
    Fresh,
    Preserve,
    NoGit,
}

fn default_git_mode() -> GitMode {
    GitMode::Fresh
}

fn default_exclude() -> Vec<String> {
    vec!["node_modules".into(), ".DS_Store".into()]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum WriteMode {
    Strict,
    NoOverwrite,
    SkipOverwrite,
    Overwrite,
    Ask,
}

fn default_write_mode() -> WriteMode {
    WriteMode::Strict
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    #[serde(default = "default_git_mode")]
    pub git: GitMode,
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
    #[serde(default = "default_write_mode")]
    pub write_mode: WriteMode,
    #[serde(default = "default_true")]
    pub color: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            version: CONFIG_VERSION,
            git: GitMode::Fresh,
            exclude: default_exclude(),
            write_mode: WriteMode::Strict,
            color: true,
        }
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let config = Self::load_from_path(&path)?;
        config.save_to_path(&path)?;
        Ok(config)
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;
        let config: Self = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse config: {}", path.display()))?;
        if config.version > CONFIG_VERSION {
            return Err(TemplativeError::UnsupportedConfigVersion.into());
        }
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        self.save_to_path(&Self::config_path()?)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let parent = path.parent().context("config path has no parent")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir: {}", parent.display()))?;
        let contents = serde_json::to_string_pretty(self).context("failed to serialize config")?;
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, &contents)
            .with_context(|| format!("failed to write config: {}", temp_path.display()))?;
        fs::rename(&temp_path, path)
            .with_context(|| format!("failed to rename config: {}", path.display()))?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        Ok(utilities::config_dir()?.join(CONFIG_FILENAME))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_file_returns_defaults() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("nonexistent.json");
        let config = Config::load_from_path(&path).unwrap();
        assert_eq!(config.version, 1);
    }

    #[test]
    fn save_then_reload_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        let config = Config::new();
        config.save_to_path(&path).unwrap();
        let loaded = Config::load_from_path(&path).unwrap();
        assert_eq!(loaded.version, config.version);
    }

    #[test]
    fn rejects_future_version() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        std::fs::write(&path, r#"{"version": 99}"#).unwrap();
        let result = Config::load_from_path(&path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<TemplativeError>(),
            Some(TemplativeError::UnsupportedConfigVersion)
        ));
    }

    #[test]
    fn accepts_current_version() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        std::fs::write(&path, r#"{"version": 1}"#).unwrap();
        let result = Config::load_from_path(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, 1);
    }

    #[test]
    fn old_config_without_git_field_defaults_to_fresh() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        std::fs::write(&path, r#"{"version": 1}"#).unwrap();
        let config = Config::load_from_path(&path).unwrap();
        assert_eq!(config.git, GitMode::Fresh);
    }

    #[test]
    fn color_defaults_to_true() {
        let json = r#"{"version":1}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.color);
    }

    #[test]
    fn color_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        let mut config = Config::new();
        config.color = false;
        config.save_to_path(&path).unwrap();
        let loaded = Config::load_from_path(&path).unwrap();
        assert!(!loaded.color);
    }

    #[test]
    fn git_mode_serializes_kebab_case() {
        let config = Config::new();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("fresh"));
    }

    #[test]
    fn git_mode_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        let config = Config {
            version: 1,
            git: GitMode::Preserve,
            exclude: vec!["dist".into()],
            write_mode: WriteMode::Strict,
            color: true,
        };
        config.save_to_path(&path).unwrap();
        let loaded = Config::load_from_path(&path).unwrap();
        assert_eq!(loaded.git, GitMode::Preserve);
    }

    #[test]
    fn git_mode_no_git_roundtrip() {
        let json = r#"{"version":1,"git":"no-git"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.git, GitMode::NoGit);
    }

    #[test]
    fn default_exclude_is_node_modules_and_ds_store() {
        let config = Config::new();
        assert_eq!(config.exclude, vec!["node_modules", ".DS_Store"]);
    }

    #[test]
    fn exclude_roundtrips_through_json() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        let mut config = Config::new();
        config.exclude = vec!["dist".into(), "*.log".into()];
        config.save_to_path(&path).unwrap();
        let loaded = Config::load_from_path(&path).unwrap();
        assert_eq!(loaded.exclude, vec!["dist", "*.log"]);
    }

    #[test]
    fn old_config_without_exclude_defaults() {
        let json = r#"{"version":1}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.exclude, vec!["node_modules", ".DS_Store"]);
    }

    #[test]
    fn default_write_mode_is_strict() {
        let config = Config::new();
        assert_eq!(config.write_mode, WriteMode::Strict);
    }

    #[test]
    fn old_config_without_write_mode_defaults_to_strict() {
        let json = r#"{"version":1}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.write_mode, WriteMode::Strict);
    }

    #[test]
    fn write_mode_roundtrip() {
        let json = r#"{"version":1,"write_mode":"skip-overwrite"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.write_mode, WriteMode::SkipOverwrite);
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("skip-overwrite"));
    }
}

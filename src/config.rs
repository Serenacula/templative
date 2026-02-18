use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::errors::TemplativeError;
use crate::utilities;

const CONFIG_VERSION: u32 = 1;
const CONFIG_FILENAME: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    // Settings added here as features land.
}

impl Config {
    pub fn new() -> Self {
        Self { version: CONFIG_VERSION }
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let config = Self::load_from_path(&path)?;
        if !path.exists() {
            config.save_to_path(&path)?;
        }
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
        let contents = serde_json::to_string_pretty(self)
            .context("failed to serialize config")?;
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
}

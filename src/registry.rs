use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::errors::TemplativeError;

const REGISTRY_VERSION: u32 = 1;
const REGISTRY_FILENAME: &str = "templates.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub templates: BTreeMap<String, String>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            version: REGISTRY_VERSION,
            templates: BTreeMap::new(),
        }
    }

    pub fn config_dir() -> Result<std::path::PathBuf> {
        let project_dirs = ProjectDirs::from("dev", "templative", "templative")
            .context("could not determine config directory")?;
        Ok(project_dirs.config_dir().to_path_buf())
    }

    pub fn registry_path() -> Result<std::path::PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join(REGISTRY_FILENAME))
    }

    pub fn load() -> Result<Self> {
        let path = Self::registry_path()?;
        let registry = Self::load_from_path(&path)?;
        if !path.exists() {
            registry.save_to_path(&path)?;
        }
        Ok(registry)
    }

    /// Load registry from a specific path (for tests).
    pub fn load_from_path(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read registry: {}", path.display()))?;
        let registry: Self = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse registry: {}", path.display()))?;
        if registry.version != REGISTRY_VERSION {
            return Err(TemplativeError::UnsupportedRegistryVersion.into());
        }
        Ok(registry)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::registry_path()?;
        self.save_to_path(&path)
    }

    /// Save registry to a specific path (for tests). Parent dir must exist.
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<()> {
        let parent = path.parent().context("registry path has no parent")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir: {}", parent.display()))?;
        let contents = serde_json::to_string_pretty(self)
            .context("failed to serialize registry")?;
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, contents)
            .with_context(|| format!("failed to write registry: {}", temp_path.display()))?;
        fs::rename(&temp_path, path)
            .with_context(|| format!("failed to rename registry: {}", path.display()))?;
        Ok(())
    }

    pub fn add(&mut self, name: String, path: String) -> Result<()> {
        if self.templates.contains_key(&name) {
            return Err(TemplativeError::TemplateExists { name }.into());
        }
        self.templates.insert(name, path);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        if !self.templates.contains_key(name) {
            return Err(TemplativeError::TemplateNotFound {
                name: name.to_string(),
            }
            .into());
        }
        self.templates.remove(name);
        Ok(())
    }

    pub fn get_path(&self, name: &str) -> Option<std::path::PathBuf> {
        self.templates.get(name).map(|string| Path::new(string).to_path_buf())
    }

    pub fn template_names_sorted(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_file_returns_empty_registry() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("nonexistent.json");
        let registry = Registry::load_from_path(&path).unwrap();
        assert_eq!(registry.version, 1);
        assert!(registry.templates.is_empty());
    }

    #[test]
    fn save_then_reload_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("templates.json");
        let mut registry = Registry::new();
        registry.templates.insert("foo".into(), "/path/to/foo".into());
        registry.save_to_path(&path).unwrap();
        let loaded = Registry::load_from_path(&path).unwrap();
        assert_eq!(loaded.templates.get("foo").unwrap(), "/path/to/foo");
    }

    #[test]
    fn rejects_version_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("templates.json");
        std::fs::write(
            &path,
            r#"{"version": 99, "templates": {}}"#,
        )
        .unwrap();
        let result = Registry::load_from_path(&path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<TemplativeError>(),
            Some(TemplativeError::UnsupportedRegistryVersion)
        ));
    }
}

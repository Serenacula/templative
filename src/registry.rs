use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{GitMode, WriteMode};
use crate::errors::TemplativeError;

const REGISTRY_VERSION: u32 = 2;
const REGISTRY_FILENAME: &str = "templates.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_init: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_init: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_mode: Option<WriteMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub templates: Vec<Template>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            version: REGISTRY_VERSION,
            templates: Vec::new(),
        }
    }

    pub fn registry_path() -> Result<PathBuf> {
        Ok(crate::utilities::config_dir()?.join(REGISTRY_FILENAME))
    }

    pub fn load() -> Result<Self> {
        let path = Self::registry_path()?;
        let registry = Self::load_from_path(&path)?;
        if !path.exists() {
            registry.save_to_path(&path)?;
        }
        Ok(registry)
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read registry: {}", path.display()))?;
        let registry: Self = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse registry: {}", path.display()))?;
        if registry.version != REGISTRY_VERSION {
            return Err(TemplativeError::UnsupportedRegistryVersion {
                found: registry.version,
                expected: REGISTRY_VERSION,
                path: path.display().to_string(),
            }
            .into());
        }
        Ok(registry)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to_path(&Self::registry_path()?)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let parent = path.parent().context("registry path has no parent")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir: {}", parent.display()))?;
        let contents =
            serde_json::to_string_pretty(self).context("failed to serialize registry")?;
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, contents)
            .with_context(|| format!("failed to write registry: {}", temp_path.display()))?;
        fs::rename(&temp_path, path)
            .with_context(|| format!("failed to rename registry: {}", path.display()))?;
        Ok(())
    }

    pub fn add(&mut self, template: Template) -> Result<()> {
        if self.templates.iter().any(|tmpl| tmpl.name == template.name) {
            return Err(TemplativeError::TemplateExists {
                name: template.name,
            }
            .into());
        }
        self.templates.push(template);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        let pos = self
            .templates
            .iter()
            .position(|tmpl| tmpl.name == name)
            .ok_or_else(|| TemplativeError::TemplateNotFound {
                name: name.to_string(),
            })?;
        self.templates.remove(pos);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.iter().find(|tmpl| tmpl.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Template> {
        self.templates.iter_mut().find(|tmpl| tmpl.name == name)
    }

    pub fn templates_sorted(&self) -> Vec<&Template> {
        let mut sorted: Vec<&Template> = self.templates.iter().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        sorted
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

    fn make_template(git: Option<GitMode>) -> Template {
        Template {
            name: "foo".into(),
            location: "/path/to/foo".into(),
            git,
            description: None,
            pre_init: None,
            post_init: None,
            git_ref: None,
            no_cache: None,
            exclude: None,
            write_mode: None,
        }
    }

    #[test]
    fn load_missing_file_returns_empty_registry() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("nonexistent.json");
        let registry = Registry::load_from_path(&path).unwrap();
        assert_eq!(registry.version, 2);
        assert!(registry.templates.is_empty());
    }

    #[test]
    fn save_then_reload_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("templates.json");
        let mut registry = Registry::new();
        registry.templates.push(make_template(None));
        registry.save_to_path(&path).unwrap();
        let loaded = Registry::load_from_path(&path).unwrap();
        assert_eq!(loaded.templates[0].location, "/path/to/foo");
    }

    #[test]
    fn rejects_version_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("templates.json");
        std::fs::write(&path, r#"{"version": 99, "templates": []}"#).unwrap();
        let result = Registry::load_from_path(&path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<TemplativeError>(),
            Some(TemplativeError::UnsupportedRegistryVersion { .. })
        ));
    }

    #[test]
    fn old_registry_without_git_field_deserializes_cleanly() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("templates.json");
        std::fs::write(
            &path,
            r#"{"version": 2, "templates": [{"name": "foo", "location": "/path"}]}"#,
        )
        .unwrap();
        let registry = Registry::load_from_path(&path).unwrap();
        let template = &registry.templates[0];
        assert!(template.git.is_none());
        assert!(template.git_ref.is_none());
        assert!(template.no_cache.is_none());
    }

    #[test]
    fn git_mode_fields_serialize_when_set() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("templates.json");
        let mut registry = Registry::new();
        registry.templates.push(Template {
            git: Some(GitMode::Preserve),
            git_ref: Some("main".into()),
            no_cache: Some(true),
            ..make_template(None)
        });
        registry.save_to_path(&path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("preserve"));
        assert!(contents.contains("git_ref"));
        assert!(contents.contains("main"));
        assert!(contents.contains("no_cache"));
    }

    #[test]
    fn git_mode_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("templates.json");
        let mut registry = Registry::new();
        registry.templates.push(make_template(Some(GitMode::NoGit)));
        registry.save_to_path(&path).unwrap();
        let loaded = Registry::load_from_path(&path).unwrap();
        assert_eq!(loaded.templates[0].git, Some(GitMode::NoGit));
    }

    #[test]
    fn skips_none_fields_in_json() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("templates.json");
        let mut registry = Registry::new();
        registry.templates.push(make_template(None));
        registry.save_to_path(&path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(!contents.contains("null"));
    }
}

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{GitMode, WriteMode};
use crate::errors::TemplativeError;
use crate::registry::Registry;

pub struct ChangeOptions {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub location: Option<PathBuf>,
    pub git: Option<Option<GitMode>>,
    pub pre_init: Option<Option<String>>,
    pub post_init: Option<Option<String>>,
    pub git_ref: Option<Option<String>>,
    pub no_cache: Option<Option<bool>>,
    pub exclude: Option<Option<Vec<String>>>,
    pub write_mode: Option<Option<WriteMode>>,
}

impl ChangeOptions {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_none()
            && self.location.is_none()
            && self.git.is_none()
            && self.pre_init.is_none()
            && self.post_init.is_none()
            && self.git_ref.is_none()
            && self.no_cache.is_none()
            && self.exclude.is_none()
            && self.write_mode.is_none()
    }
}

pub fn cmd_change(template_name: String, options: ChangeOptions) -> Result<()> {
    if options.is_empty() {
        anyhow::bail!("no changes specified");
    }

    let mut registry = Registry::load()?;

    if registry.get(&template_name).is_none() {
        return Err(TemplativeError::TemplateNotFound { name: template_name.clone() }.into());
    }
    if let Some(ref new_name) = options.name {
        if registry.get(new_name).is_some() {
            return Err(TemplativeError::TemplateExists { name: new_name.clone() }.into());
        }
    }

    let template = registry.get_mut(&template_name).unwrap();

    if let Some(new_name) = options.name { template.name = new_name; }
    if let Some(new_description) = options.description { template.description = new_description; }
    if let Some(new_git) = options.git { template.git = new_git; }
    if let Some(new_location) = options.location {
        let canonical = new_location
            .canonicalize()
            .with_context(|| format!("path not found: {}", new_location.display()))?;
        template.location = canonical.to_string_lossy().into_owned();
    }
    if let Some(new_pre_init) = options.pre_init { template.pre_init = new_pre_init; }
    if let Some(new_post_init) = options.post_init { template.post_init = new_post_init; }
    if let Some(new_git_ref) = options.git_ref { template.git_ref = new_git_ref; }
    if let Some(new_no_cache) = options.no_cache { template.no_cache = new_no_cache; }
    if let Some(new_exclude) = options.exclude { template.exclude = new_exclude; }
    if let Some(new_write_mode) = options.write_mode { template.write_mode = new_write_mode; }

    registry.save()?;
    println!("updated {}", template_name);
    Ok(())
}

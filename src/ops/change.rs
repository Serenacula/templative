use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::GitMode;
use crate::errors::TemplativeError;
use crate::registry::Registry;

pub fn cmd_change(
    template_name: String,
    name: Option<String>,
    description: Option<String>,
    location: Option<PathBuf>,
    git: Option<Option<GitMode>>,
    commit: Option<String>,
    pre_init: Option<String>,
    post_init: Option<String>,
    git_ref: Option<String>,
    no_cache: Option<Option<bool>>,
) -> Result<()> {
    if name.is_none() && description.is_none() && location.is_none()
        && git.is_none() && commit.is_none() && pre_init.is_none() && post_init.is_none()
        && git_ref.is_none() && no_cache.is_none()
    {
        anyhow::bail!("no changes specified");
    }

    let mut registry = Registry::load()?;

    if let Some(ref new_name) = name {
        if registry.get(new_name).is_some() {
            return Err(TemplativeError::TemplateExists { name: new_name.clone() }.into());
        }
    }

    let template = registry
        .get_mut(&template_name)
        .ok_or_else(|| TemplativeError::TemplateNotFound { name: template_name.clone() })?;

    if let Some(new_name) = name { template.name = new_name; }
    if let Some(new_description) = description { template.description = Some(new_description); }
    if let Some(new_git) = git { template.git = new_git; }
    if let Some(new_location) = location {
        let canonical = new_location
            .canonicalize()
            .with_context(|| format!("path not found: {}", new_location.display()))?;
        template.location = canonical.to_string_lossy().into_owned();
    }
    if let Some(new_commit) = commit { template.commit = Some(new_commit); }
    if let Some(new_pre_init) = pre_init { template.pre_init = Some(new_pre_init); }
    if let Some(new_post_init) = post_init { template.post_init = Some(new_post_init); }
    if let Some(new_git_ref) = git_ref { template.git_ref = Some(new_git_ref); }
    if let Some(new_no_cache) = no_cache { template.no_cache = new_no_cache; }

    registry.save()?;
    println!("updated {}", template_name);
    Ok(())
}

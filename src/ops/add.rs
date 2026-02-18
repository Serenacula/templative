use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::GitMode;
use crate::git_cache;
use crate::registry::{Registry, Template};
use crate::utilities;

pub fn cmd_add(
    path: String,
    name: Option<String>,
    description: Option<String>,
    git: Option<GitMode>,
    git_ref: Option<String>,
    no_cache: Option<bool>,
) -> Result<()> {
    let (location, template_name) = if utilities::is_git_url(&path) {
        git_cache::ensure_cached(&path)?;
        let name = name.unwrap_or_else(|| {
            path.trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("template")
                .trim_end_matches(".git")
                .to_string()
        });
        (path, name)
    } else {
        let canonical = PathBuf::from(&path)
            .canonicalize()
            .with_context(|| format!("path not found or not absolute: {}", path))?;
        let name = name.unwrap_or_else(|| {
            canonical
                .file_name()
                .map(|os| os.to_string_lossy().into_owned())
                .unwrap_or_else(|| "template".to_string())
        });
        (canonical.to_string_lossy().into_owned(), name)
    };

    let template = Template {
        name: template_name.clone(),
        location: location.clone(),
        git,
        description,
        commit: None,
        pre_init: None,
        post_init: None,
        git_ref,
        no_cache,
    };
    let mut registry = Registry::load()?;
    registry.add(template)?;
    registry.save()?;
    println!("added {} -> {}", template_name, location);
    Ok(())
}

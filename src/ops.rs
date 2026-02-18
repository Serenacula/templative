use std::path::PathBuf;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;

use crate::config::Config;
use crate::errors::TemplativeError;
use crate::fs_copy;
use crate::git;
use crate::registry::{Registry, Template};
use crate::utilities;

pub fn cmd_add(_config: Config, path: PathBuf, name: Option<String>, description: Option<String>) -> Result<()> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("path not found or not absolute: {}", path.display()))?;
    let template_name = name.unwrap_or_else(|| {
        canonical
            .file_name()
            .map(|os| os.to_string_lossy().into_owned())
            .unwrap_or_else(|| "template".to_string())
    });
    let location = canonical.to_string_lossy().into_owned();
    let template = Template {
        name: template_name.clone(),
        location: location.clone(),
        description,
        commit: None,
        pre_init: None,
        post_init: None,
    };
    let mut registry = Registry::load()?;
    registry.add(template)?;
    registry.save()?;
    println!("added {} -> {}", template_name, location);
    Ok(())
}

pub fn cmd_remove(_config: Config, template_name: String) -> Result<()> {
    let mut registry = Registry::load()?;
    registry.remove(&template_name)?;
    registry.save()?;
    println!("removed {}", template_name);
    Ok(())
}

pub fn cmd_list(_config: Config) -> Result<()> {
    let registry = Registry::load()?;
    if registry.templates.is_empty() {
        println!("no templates available: use `templative add <FOLDER>` to add a template");
        return Ok(());
    }
    for template in registry.templates_sorted() {
        let path_buf = PathBuf::from(&template.location);
        if path_buf.exists() {
            println!("{}  {}", template.name, template.location);
        } else {
            let display = format!("{}  {} (missing)", template.name, template.location);
            println!("{}", display.strikethrough().red());
        }
    }
    Ok(())
}

pub fn cmd_init(_config: Config, template_name: String, target_path: PathBuf) -> Result<()> {
    let registry = Registry::load()?;
    let template = registry
        .get(&template_name)
        .ok_or_else(|| TemplativeError::TemplateNotFound {
            name: template_name.clone(),
        })
        .with_context(|| "run 'templative list' to see available templates")?;

    let template_path = PathBuf::from(&template.location);

    if !template_path.exists() {
        return Err(TemplativeError::TemplatePathMissing {
            path: template_path.clone(),
        }
        .into());
    }
    if !template_path.is_dir() {
        return Err(TemplativeError::TemplatePathMissing {
            path: template_path.clone(),
        }
        .into());
    }

    if !target_path.exists() {
        std::fs::create_dir_all(&target_path)
            .with_context(|| format!("failed to create target: {}", target_path.display()))?;
    }
    let target_canonical = target_path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize target: {}", target_path.display()))?;

    if utilities::is_dangerous_path(&target_canonical) {
        return Err(TemplativeError::DangerousPath {
            path: target_canonical,
        }
        .into());
    }

    if !utilities::is_dir_empty(&target_canonical)? {
        return Err(TemplativeError::TargetNotEmpty.into());
    }

    fs_copy::copy_template(&template_path, &target_canonical)?;
    git::init_and_commit(&target_canonical, &template_name)?;

    println!(
        "created {} from {}",
        target_canonical.display(),
        template_name
    );
    Ok(())
}

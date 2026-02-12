use std::path::PathBuf;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;

use crate::errors::TemplativeError;
use crate::fs_copy;
use crate::git;
use crate::registry::Registry;

fn is_dangerous_path(path: &std::path::Path) -> bool {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    path == std::path::Path::new("/")
        || home.as_deref().map_or(false, |home_dir| path == home_dir)
}

fn is_dir_empty(path: &std::path::Path) -> Result<bool> {
    let mut entries = std::fs::read_dir(path)
        .with_context(|| format!("failed to read directory: {}", path.display()))?;
    Ok(entries.next().is_none())
}

pub fn cmd_add(path: PathBuf, name: Option<String>) -> Result<()> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("path not found or not absolute: {}", path.display()))?;
    let template_name = name.unwrap_or_else(|| {
        canonical
            .file_name()
            .map(|os| os.to_string_lossy().into_owned())
            .unwrap_or_else(|| "template".to_string())
    });
    let path_str = canonical.to_string_lossy().into_owned();
    let mut registry = Registry::load()?;
    registry.add(template_name.clone(), path_str.clone())?;
    registry.save()?;
    println!("added {} -> {}", template_name, path_str);
    Ok(())
}

pub fn cmd_remove(template_name: String) -> Result<()> {
    let mut registry = Registry::load()?;
    registry.remove(&template_name)?;
    registry.save()?;
    println!("removed {}", template_name);
    Ok(())
}

pub fn cmd_list() -> Result<()> {
    let registry = Registry::load()?;
    for name in registry.template_names_sorted() {
        let path_str = registry.templates.get(&name).unwrap();
        let path_buf = PathBuf::from(path_str);
        if path_buf.exists() {
            println!("{}  {}", name, path_str);
        } else {
            let name_display = format!("{}  {} (missing)", name, path_str);
            println!("{}", name_display.strikethrough().red());
        }
    }
    Ok(())
}

pub fn cmd_init(template_name: String, target_path: PathBuf) -> Result<()> {
    let registry = Registry::load()?;
    let template_path = registry
        .get_path(&template_name)
        .ok_or_else(|| TemplativeError::TemplateNotFound { name: template_name.clone() })?;
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

    if is_dangerous_path(&target_canonical) {
        return Err(TemplativeError::DangerousPath {
            path: target_canonical,
        }
        .into());
    }

    if !is_dir_empty(&target_canonical)? {
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

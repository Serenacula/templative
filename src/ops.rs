use std::path::PathBuf;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;

use unicode_width::UnicodeWidthStr;

use crate::config::{Config, GitMode, UpdateOnInit};
use crate::errors::TemplativeError;
use crate::fs_copy;
use crate::git;
use crate::git_cache;
use crate::registry::{Registry, Template};
use crate::resolved::ResolvedOptions;
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

pub fn cmd_remove(template_name: String) -> Result<()> {
    let mut registry = Registry::load()?;
    registry.remove(&template_name)?;
    registry.save()?;
    println!("removed {}", template_name);
    Ok(())
}

pub fn cmd_list() -> Result<()> {
    let registry = Registry::load()?;
    if registry.templates.is_empty() {
        println!("no templates available: use `templative add <FOLDER>` to add a template");
        return Ok(());
    }
    let templates = registry.templates_sorted();

    let name_w = templates.iter()
        .map(|t| t.name.width())
        .max().unwrap_or(0)
        .max("NAME".width());
    let desc_w = templates.iter()
        .map(|t| t.description.as_deref().unwrap_or("").width())
        .max().unwrap_or(0)
        .max("DESCRIPTION".width());

    let pad = |s: &str, w: usize| -> String {
        let display_w = s.width();
        format!("{}{}", s, " ".repeat(w.saturating_sub(display_w)))
    };

    println!("{}  {}  {}",
        pad("NAME", name_w).underline(),
        pad("DESCRIPTION", desc_w).underline(),
        "LOCATION".underline());

    for template in templates {
        let desc = template.description.as_deref().unwrap_or("");
        let is_missing = !utilities::is_git_url(&template.location)
            && !PathBuf::from(&template.location).exists();
        let location = if is_missing {
            format!("{} (missing)", template.location)
        } else {
            template.location.clone()
        };
        let row = format!("{}  {}  {}", pad(&template.name, name_w), pad(desc, desc_w), location);
        if is_missing {
            println!("{}", row.strikethrough().red());
        } else {
            println!("{}", row);
        }
    }
    Ok(())
}

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

pub fn cmd_init(
    config: Config,
    template_name: String,
    target_path: PathBuf,
    git_flag: Option<GitMode>,
) -> Result<()> {
    let registry = Registry::load()?;
    let template = registry
        .get(&template_name)
        .ok_or_else(|| TemplativeError::TemplateNotFound {
            name: template_name.clone(),
        })
        .with_context(|| "run 'templative list' to see available templates")?;

    let resolved = ResolvedOptions::build(&config, template, git_flag);
    let location = template.location.clone();
    let location_is_url = utilities::is_git_url(&location);

    // Determine template source path (and keep tempdir alive if used)
    let _tempdir: Option<tempfile::TempDir>;
    let template_path: PathBuf;

    if location_is_url {
        if resolved.no_cache {
            let td = tempfile::tempdir().context("failed to create temp dir")?;
            git::clone_repo(&location, td.path())?;
            if let Some(ref git_ref) = resolved.git_ref {
                git::checkout_ref(td.path(), git_ref)?;
            }
            template_path = td.path().to_path_buf();
            _tempdir = Some(td);
        } else {
            let cache_path = git_cache::ensure_cached(&location)?;
            let should_update = resolved.git_ref.is_none()
                && resolved.update_on_init != UpdateOnInit::Never;
            if should_update {
                let _ = git_cache::update_cache(&cache_path);
            }
            if let Some(ref git_ref) = resolved.git_ref {
                git::checkout_ref(&cache_path, git_ref)?;
            }
            template_path = cache_path;
            _tempdir = None;
        }
    } else {
        template_path = PathBuf::from(&location);
        _tempdir = None;
        let git_dir = template_path.join(".git");
        if resolved.git_ref.is_none()
            && resolved.update_on_init == UpdateOnInit::Always
            && git_dir.exists()
        {
            let _ = git::fetch_origin(&template_path);
            let _ = git::reset_hard_origin(&template_path);
        }
    }

    if !template_path.exists() || !template_path.is_dir() {
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

    if let Some(ref cmd) = resolved.pre_init {
        utilities::run_hook(cmd, &target_canonical)?;
    }

    if !utilities::is_dir_empty(&target_canonical)? {
        return Err(TemplativeError::TargetNotEmpty.into());
    }

    match resolved.git {
        GitMode::Fresh => {
            fs_copy::copy_template(&template_path, &target_canonical)?;
            git::init_and_commit(&target_canonical, &template_name)?;
        }
        GitMode::Preserve => {
            git::clone_local(&template_path, &target_canonical)?;
            if location_is_url {
                git::set_remote_url(&target_canonical, &location)?;
            }
        }
        GitMode::NoGit => {
            fs_copy::copy_template(&template_path, &target_canonical)?;
        }
    }

    if let Some(ref cmd) = resolved.post_init {
        if let Err(err) = utilities::run_hook(cmd, &target_canonical) {
            eprintln!("warning: post-init hook failed: {:#}", err);
        }
    }

    println!(
        "created {} from {}",
        target_canonical.display(),
        template_name
    );
    Ok(())
}

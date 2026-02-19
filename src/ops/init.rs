use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{Config, GitMode, UpdateOnInit, WriteMode};
use crate::errors::TemplativeError;
use crate::fs_copy;
use crate::git;
use crate::git_cache;
use crate::registry::Registry;
use crate::resolved::ResolvedOptions;
use crate::utilities;

/// Resolves the template source path, cloning or using cache as needed.
/// Returns the path and an optional TempDir that must stay alive for the duration of the copy.
fn resolve_template_path(
    location: &str,
    location_is_url: bool,
    resolved: &ResolvedOptions,
) -> Result<(PathBuf, Option<tempfile::TempDir>)> {
    if location_is_url {
        if resolved.no_cache {
            let tempdir = tempfile::tempdir().context("failed to create temp dir")?;
            git::clone_repo(location, tempdir.path())?;
            if let Some(ref git_ref) = resolved.git_ref {
                git::checkout_ref(tempdir.path(), git_ref)?;
            }
            let path = tempdir.path().to_path_buf();
            Ok((path, Some(tempdir)))
        } else {
            let cache_path = git_cache::ensure_cached(location)?;
            let should_update = resolved.update_on_init != UpdateOnInit::Never;
            if should_update {
                git_cache::update_cache(&cache_path);
            }
            if let Some(ref git_ref) = resolved.git_ref {
                git::checkout_ref(&cache_path, git_ref)?;
            }
            Ok((cache_path, None))
        }
    } else {
        let path = PathBuf::from(location);
        let git_dir = path.join(".git");
        if resolved.git_ref.is_none()
            && resolved.update_on_init == UpdateOnInit::Always
            && git_dir.exists()
        {
            let _ = git::fetch_origin(&path);
            let _ = git::reset_hard_origin(&path);
        }
        Ok((path, None))
    }
}

pub fn cmd_init(
    config: Config,
    template_name: String,
    target_path: PathBuf,
    git_flag: Option<GitMode>,
    write_mode_flag: Option<WriteMode>,
) -> Result<()> {
    let registry = Registry::load()?;
    let template = registry
        .get(&template_name)
        .ok_or_else(|| TemplativeError::TemplateNotFound {
            name: template_name.clone(),
        })
        .with_context(|| "run 'templative list' to see available templates")?;

    let resolved = ResolvedOptions::build(&config, template, git_flag, write_mode_flag);
    let location = template.location.clone();
    let location_is_url = utilities::is_git_url(&location);

    let (template_path, _tempdir) = resolve_template_path(&location, location_is_url, &resolved)?;

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

    if resolved.write_mode == WriteMode::Strict && !utilities::is_dir_empty(&target_canonical)? {
        return Err(TemplativeError::TargetNotEmpty.into());
    }

    match resolved.git {
        GitMode::Fresh => {
            fs_copy::copy_template(
                &template_path,
                &target_canonical,
                &resolved.exclude,
                &resolved.write_mode,
            )?;
            if target_canonical.join(".git").exists() {
                git::add_and_commit(&target_canonical, &template_name)?;
            } else {
                git::init_and_commit(&target_canonical, &template_name)?;
            }
        }
        GitMode::Preserve => {
            git::clone_local(&template_path, &target_canonical)?;
            if location_is_url {
                git::set_remote_url(&target_canonical, &location)?;
            }
        }
        GitMode::NoGit => {
            fs_copy::copy_template(
                &template_path,
                &target_canonical,
                &resolved.exclude,
                &resolved.write_mode,
            )?;
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

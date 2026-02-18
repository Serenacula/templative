use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{Config, GitMode, UpdateOnInit};
use crate::errors::TemplativeError;
use crate::fs_copy;
use crate::git;
use crate::git_cache;
use crate::registry::Registry;
use crate::resolved::ResolvedOptions;
use crate::utilities;

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

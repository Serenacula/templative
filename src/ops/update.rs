use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::errors::TemplativeError;
use crate::git::{self, RefKind};
use crate::git_cache;
use crate::registry::{Registry, Template};
use crate::utilities;

pub fn cmd_update(template_name: Option<String>, check: bool) -> Result<()> {
    let registry = Registry::load()?;

    let templates: Vec<Template> = if let Some(ref name) = template_name {
        let tmpl = registry
            .get(name)
            .ok_or_else(|| TemplativeError::TemplateNotFound { name: name.clone() })?;
        vec![tmpl.clone()]
    } else {
        registry.templates_sorted().into_iter().cloned().collect()
    };

    if templates.is_empty() {
        println!("no templates registered");
        return Ok(());
    }

    let mut errors: Vec<String> = Vec::new();
    for tmpl in &templates {
        match update_template(tmpl, check) {
            Ok(status) => println!("{}: {}", tmpl.name, status),
            Err(err) => errors.push(format!("{}: {:#}", tmpl.name, err)),
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("some templates failed to update:\n{}", errors.join("\n"));
    }

    Ok(())
}

pub(crate) fn update_template(tmpl: &Template, check: bool) -> Result<String> {
    if utilities::is_git_url(&tmpl.location) {
        update_url_template(tmpl, check)
    } else {
        update_local_template(tmpl, check)
    }
}

fn update_url_template(tmpl: &Template, check: bool) -> Result<String> {
    let cache_path = utilities::cache_path_for_url(&tmpl.location)?;
    if !cache_path.exists() {
        git_cache::ensure_cached(&tmpl.location)?;
    }
    git::fetch_origin(&cache_path).context("fetch failed")?;
    if check {
        return Ok(if git::is_behind_remote(&cache_path) {
            "update available".into()
        } else {
            "up to date".into()
        });
    }
    if let Some(ref git_ref) = tmpl.git_ref {
        match git::classify_ref(&cache_path, git_ref) {
            RefKind::Branch => {
                git::checkout_ref(&cache_path, git_ref)?;
                Ok("updated".into())
            }
            RefKind::Tag | RefKind::Commit => Ok("skipped (pinned to immutable ref)".into()),
        }
    } else {
        git::reset_hard_origin(&cache_path).context("reset failed")?;
        Ok("updated".into())
    }
}

fn update_local_template(tmpl: &Template, check: bool) -> Result<String> {
    let path = PathBuf::from(&tmpl.location);
    if !git::is_git_repo(&path) {
        return Ok("skipped (not a git repository)".into());
    }
    // Fetch is non-fatal: no remote configured is fine
    let _ = git::fetch_origin(&path);
    if check {
        return Ok(if git::is_behind_remote(&path) {
            "update available".into()
        } else {
            "up to date".into()
        });
    }
    if let Some(ref git_ref) = tmpl.git_ref {
        match git::classify_ref(&path, git_ref) {
            RefKind::Branch => {
                git::checkout_ref(&path, git_ref)?;
                Ok("updated".into())
            }
            RefKind::Tag | RefKind::Commit => Ok("skipped (pinned to immutable ref)".into()),
        }
    } else {
        git::pull_ff_only(&path).context("pull failed")?;
        Ok("updated".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_template(name: &str, location: &str) -> Template {
        Template {
            name: name.into(),
            location: location.into(),
            git: None,
            description: None,
            pre_init: None,
            post_init: None,
            git_ref: None,
            exclude: None,
            write_mode: None,
        }
    }

    fn git(dir: &std::path::Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .status()
            .unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    fn setup_repo(dir: &std::path::Path) {
        git(dir, &["init"]);
        std::fs::write(dir.join("file.txt"), "v1").unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-m", "initial"]);
    }

    #[test]
    fn local_non_git_dir_is_skipped() {
        let dir = tempdir().unwrap();
        let tmpl = make_template("test", dir.path().to_str().unwrap());
        let result = update_template(&tmpl, false).unwrap();
        assert_eq!(result, "skipped (not a git repository)");
    }

    #[test]
    fn local_non_git_dir_check_is_skipped() {
        let dir = tempdir().unwrap();
        let tmpl = make_template("test", dir.path().to_str().unwrap());
        let result = update_template(&tmpl, true).unwrap();
        assert_eq!(result, "skipped (not a git repository)");
    }

    #[test]
    fn local_git_no_remote_check_returns_up_to_date() {
        let dir = tempdir().unwrap();
        setup_repo(dir.path());
        let tmpl = make_template("test", dir.path().to_str().unwrap());
        let result = update_template(&tmpl, true).unwrap();
        assert_eq!(result, "up to date");
    }

    #[test]
    fn local_git_behind_remote_check_returns_update_available() {
        let remote = tempdir().unwrap();
        setup_repo(remote.path());
        let local = tempdir().unwrap();
        git(
            local.path().parent().unwrap(),
            &[
                "clone",
                remote.path().to_str().unwrap(),
                local.path().to_str().unwrap(),
            ],
        );
        // Add a commit to remote
        std::fs::write(remote.path().join("file.txt"), "v2").unwrap();
        git(remote.path(), &["add", "-A"]);
        git(remote.path(), &["commit", "-m", "update"]);

        let tmpl = make_template("test", local.path().to_str().unwrap());
        let result = update_template(&tmpl, true).unwrap();
        assert_eq!(result, "update available");
    }

    #[test]
    fn local_git_pinned_tag_is_skipped() {
        let dir = tempdir().unwrap();
        setup_repo(dir.path());
        git(dir.path(), &["tag", "v1.0"]);
        let mut tmpl = make_template("test", dir.path().to_str().unwrap());
        tmpl.git_ref = Some("v1.0".into());
        let result = update_template(&tmpl, false).unwrap();
        assert_eq!(result, "skipped (pinned to immutable ref)");
    }
}

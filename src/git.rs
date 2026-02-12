use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

fn run_git(target_path: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(target_path)
        .output()
        .context("failed to execute git")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr);
    }
    Ok(())
}

pub fn init_repo(target_path: &Path) -> Result<()> {
    run_git(target_path, &["init"]).context("git init failed")
}

pub fn add_all(target_path: &Path) -> Result<()> {
    run_git(target_path, &["add", "-A"]).context("git add -A failed")
}

pub fn initial_commit(target_path: &Path, template_name: &str) -> Result<()> {
    let message = format!("Initial commit from template: {}", template_name);
    run_git(target_path, &["commit", "-m", &message]).context("git commit failed")
}

pub fn init_and_commit(target_path: &Path, template_name: &str) -> Result<()> {
    init_repo(target_path)?;
    add_all(target_path)?;
    initial_commit(target_path, template_name)?;
    Ok(())
}

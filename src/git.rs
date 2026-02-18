use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

fn git_config_get(key: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["config", key])
        .output()
        .context("failed to execute git")?;
    if output.status.code() == Some(1) {
        return Ok(String::new());
    }
    if !output.status.success() {
        anyhow::bail!("git config {} failed", key);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn check_user_config() -> Result<()> {
    let name = git_config_get("user.name")?;
    let email = git_config_get("user.email")?;
    let mut missing = Vec::new();
    if name.is_empty() {
        missing.push("  git config --global user.name \"Your Name\"");
    }
    if email.is_empty() {
        missing.push("  git config --global user.email \"you@example.com\"");
    }
    if !missing.is_empty() {
        anyhow::bail!("git identity not set; run:\n{}", missing.join("\n"));
    }
    Ok(())
}

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
    check_user_config()?;
    init_repo(target_path)?;
    add_all(target_path)?;
    initial_commit(target_path, template_name)?;
    Ok(())
}

use std::path::PathBuf;

use anyhow::{Context, Result};

#[cfg(not(unix))]
use directories::ProjectDirs;

pub fn config_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(|home| PathBuf::from(home).join(".config"))
            });
        match base {
            Some(path) => Ok(path.join("templative")),
            None => Err(anyhow::anyhow!(
                "could not determine config directory (set HOME or XDG_CONFIG_HOME)"
            )),
        }
    }
    #[cfg(not(unix))]
    {
        let project_dirs = ProjectDirs::from("com", "fayleemb", "templative")
            .context("could not determine config directory")?;
        Ok(project_dirs.config_dir().to_path_buf())
    }
}

pub fn is_dangerous_path(path: &std::path::Path) -> bool {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    path == std::path::Path::new("/") || home.as_deref().map_or(false, |home_dir| path == home_dir)
}

pub fn run_hook(command: &str, working_dir: &std::path::Path) -> Result<()> {
    let output = std::process::Command::new("sh")
        .args(["-c", command])
        .current_dir(working_dir)
        .output()
        .context("failed to execute hook")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("hook failed: {}", stderr.trim());
    }
    Ok(())
}

pub fn is_dir_empty(path: &std::path::Path) -> Result<bool> {
    let mut entries = std::fs::read_dir(path)
        .with_context(|| format!("failed to read directory: {}", path.display()))?;
    Ok(entries.next().is_none())
}

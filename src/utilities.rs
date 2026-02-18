use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn is_dangerous_path(path: &std::path::Path) -> bool {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    path == std::path::Path::new("/") || home.as_deref().map_or(false, |home_dir| path == home_dir)
}

pub fn is_dir_empty(path: &std::path::Path) -> Result<bool> {
    let mut entries = std::fs::read_dir(path)
        .with_context(|| format!("failed to read directory: {}", path.display()))?;
    Ok(entries.next().is_none())
}

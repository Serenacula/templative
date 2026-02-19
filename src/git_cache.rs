use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{git, utilities};

/// Returns the cache path, cloning from the URL if not already present.
pub fn ensure_cached(url: &str) -> Result<PathBuf> {
    let cache_path = utilities::cache_path_for_url(url)?;
    if !cache_path.exists() {
        git::clone_repo(url, &cache_path)?;
    }
    Ok(cache_path)
}

/// Fetch and attempt reset to origin/HEAD. Fully non-fatal: network or ref errors are ignored.
pub fn update_cache(cache_path: &Path) {
    let _ = git::fetch_origin(cache_path);
    let _ = git::reset_hard_origin(cache_path);
}

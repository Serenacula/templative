use std::path::PathBuf;

use anyhow::{Context, Result};

#[cfg(not(unix))]
use directories::ProjectDirs;

pub fn config_dir() -> Result<PathBuf> {
    if let Some(override_dir) = std::env::var_os("TEMPLATIVE_CONFIG_DIR") {
        return Ok(PathBuf::from(override_dir));
    }
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

pub fn is_git_url(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("http://")
        || url.starts_with("git@")
        || url.starts_with("git://")
}

fn fnv1a_hash(input: &str) -> u64 {
    // FNV-1a 64-bit: standard constants from https://www.isthe.com/chongo/tech/comp/fnv/
    const OFFSET_BASIS: u64 = 14695981039346656037; // 0xcbf29ce484222325
    const PRIME: u64 = 1099511628211;               // 0x00000100000001b3
    let mut hash = OFFSET_BASIS;
    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

pub fn cache_path_for_url(url: &str) -> Result<PathBuf> {
    Ok(config_dir()?.join("cache").join(format!("{:016x}", fnv1a_hash(url))))
}

pub fn is_dir_empty(path: &std::path::Path) -> Result<bool> {
    let mut entries = std::fs::read_dir(path)
        .with_context(|| format!("failed to read directory: {}", path.display()))?;
    Ok(entries.next().is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_git_url_recognises_https() {
        assert!(is_git_url("https://github.com/user/repo"));
    }

    #[test]
    fn is_git_url_recognises_http() {
        assert!(is_git_url("http://example.com/repo"));
    }

    #[test]
    fn is_git_url_recognises_git_at() {
        assert!(is_git_url("git@github.com:user/repo.git"));
    }

    #[test]
    fn is_git_url_recognises_git_protocol() {
        assert!(is_git_url("git://example.com/repo"));
    }

    #[test]
    fn is_git_url_rejects_local_paths() {
        assert!(!is_git_url("/path/to/template"));
        assert!(!is_git_url("./relative"));
        assert!(!is_git_url("template"));
    }

    #[test]
    fn cache_path_for_url_is_deterministic() {
        let path1 = cache_path_for_url("https://github.com/user/repo").unwrap();
        let path2 = cache_path_for_url("https://github.com/user/repo").unwrap();
        assert_eq!(path1, path2);
    }

    #[test]
    fn cache_path_for_url_differs_for_different_urls() {
        let path1 = cache_path_for_url("https://github.com/user/repo-a").unwrap();
        let path2 = cache_path_for_url("https://github.com/user/repo-b").unwrap();
        assert_ne!(path1, path2);
    }

    #[test]
    fn cache_path_for_url_ends_with_hex_segment() {
        let path = cache_path_for_url("https://github.com/user/repo").unwrap();
        let hex = path.file_name().unwrap().to_string_lossy();
        assert_eq!(hex.len(), 16);
        assert!(hex.chars().all(|character| character.is_ascii_hexdigit()));
    }
}

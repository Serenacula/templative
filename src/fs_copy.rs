use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use walkdir::{DirEntry, WalkDir};

use crate::errors::TemplativeError;

const EXCLUDE_DIRS: &[&str] = &[".git", "node_modules"];
const EXCLUDE_FILES: &[&str] = &[".DS_Store"];

/// Returns true if this entry (or its path) should be skipped (excluded). Does not check symlinks.
fn should_skip_entry(entry: &DirEntry, source_root: &Path) -> bool {
    let relative = match entry.path().strip_prefix(source_root) {
        Ok(rel) => rel,
        Err(_) => return false,
    };
    for component in relative.components() {
        let part = component.as_os_str().to_string_lossy();
        if EXCLUDE_DIRS.contains(&part.as_ref()) {
            return true;
        }
    }
    if entry.file_type().is_file() {
        let name = entry.file_name().to_string_lossy();
        if EXCLUDE_FILES.contains(&name.as_ref()) {
            return true;
        }
    }
    false
}

/// Copy template from source_dir to dest_dir.
/// Excludes .git, node_modules, .DS_Store. Errors on symlinks.
/// Preserves file permissions.
pub fn copy_template(source_dir: &Path, dest_dir: &Path) -> Result<()> {
    if !source_dir.is_dir() {
        anyhow::bail!("source is not a directory: {}", source_dir.display());
    }
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("failed to create destination: {}", dest_dir.display()))?;

    let walker = WalkDir::new(source_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            let path = entry.path();
            if path == source_dir {
                return true;
            }
            !should_skip_entry(entry, source_dir)
        });

    for entry in walker {
        let entry = entry.with_context(|| "walkdir entry error")?;
        let path = entry.path();
        if path == source_dir {
            continue;
        }
        if entry.path().is_symlink() {
            return Err(TemplativeError::SymlinkNotSupported.into());
        }
        let relative = path
            .strip_prefix(source_dir)
            .with_context(|| "strip_prefix")?;
        let dest_path = dest_dir.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)
                .with_context(|| format!("failed to create dir: {}", dest_path.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create parent: {}", parent.display()))?;
            }
            fs::copy(path, &dest_path)
                .with_context(|| format!("failed to copy {} -> {}", path.display(), dest_path.display()))?;
            if let Ok(metadata) = fs::metadata(path) {
                let _ = fs::set_permissions(&dest_path, metadata.permissions());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn create_template_structure(dir: &Path) {
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::create_dir_all(dir.join("node_modules")).unwrap();
        fs::create_dir_all(dir.join("deep/nested")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        fs::write(dir.join(".git/config"), "[core]\n").unwrap();
        fs::write(dir.join("node_modules/dummy"), "").unwrap();
        fs::write(dir.join(".DS_Store"), "").unwrap();
        fs::write(dir.join("deep/nested/file.txt"), "hello").unwrap();
    }

    #[test]
    fn copies_nested_structure_and_excludes_dirs_and_ds_store() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        create_template_structure(&source);

        copy_template(&source, &dest).unwrap();

        assert!(dest.join("src/main.rs").exists());
        assert!(dest.join("Cargo.toml").exists());
        assert!(dest.join("deep/nested/file.txt").exists());
        assert_eq!(fs::read_to_string(dest.join("deep/nested/file.txt")).unwrap(), "hello");
        assert!(!dest.join(".git").exists());
        assert!(!dest.join("node_modules").exists());
        assert!(!dest.join(".DS_Store").exists());
    }

    #[test]
    fn errors_on_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(source.join("file.txt"), source.join("link.txt")).unwrap();

        let result = copy_template(&source, &dest);
        #[cfg(unix)]
        assert!(result.is_err());
        #[cfg(unix)]
        assert!(matches!(
            result.unwrap_err().downcast_ref::<TemplativeError>(),
            Some(TemplativeError::SymlinkNotSupported)
        ));
        #[cfg(not(unix))]
        let _ = result;
    }
}

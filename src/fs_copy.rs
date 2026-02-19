use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use dialoguer::Select;
use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::{DirEntry, WalkDir};

use crate::config::WriteMode;
use crate::errors::TemplativeError;

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(
            Glob::new(pattern)
                .with_context(|| format!("invalid exclude pattern: {}", pattern))?,
        );
    }
    builder.build().context("failed to build exclude patterns")
}

/// Returns true if this entry (or its path) should be skipped.
/// `.git` is always excluded. Each path component and the full relative path
/// are checked against `globset`.
fn should_skip_entry(entry: &DirEntry, source_root: &Path, globset: &GlobSet) -> bool {
    let relative = match entry.path().strip_prefix(source_root) {
        Ok(rel) => rel,
        Err(_) => return false,
    };
    for component in relative.components() {
        let part = component.as_os_str().to_string_lossy();
        if part == ".git" {
            return true;
        }
        if globset.is_match(part.as_ref()) {
            return true;
        }
    }
    if globset.is_match(relative) {
        return true;
    }
    false
}

enum FileChoice {
    Overwrite,
    Skip,
    OverwriteAll,
    SkipAll,
    Abort,
}

fn prompt_file(dest_path: &Path) -> Result<FileChoice> {
    let prompt = format!("'{}' already exists. What would you like to do?", dest_path.display());
    let options = &["Overwrite", "Skip", "Overwrite all", "Skip all", "Abort"];
    let selection = Select::new()
        .with_prompt(&prompt)
        .items(options)
        .default(0)
        .interact()
        .context("prompt failed")?;
    Ok(match selection {
        0 => FileChoice::Overwrite,
        1 => FileChoice::Skip,
        2 => FileChoice::OverwriteAll,
        3 => FileChoice::SkipAll,
        _ => FileChoice::Abort,
    })
}

/// Computes a relative path from `from_dir` to `to`. Both must be absolute.
fn relative_path_between(from_dir: &Path, to: &Path) -> PathBuf {
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to.components().collect();
    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut result = PathBuf::new();
    for _ in 0..(from_components.len() - common_len) {
        result.push("..");
    }
    for component in &to_components[common_len..] {
        result.push(component.as_os_str());
    }
    if result.as_os_str().is_empty() {
        result.push(".");
    }
    result
}

/// Copies a symlink from `source_path` to `dest_path`, adjusting the target:
/// - If target resolves inside the template, keeps a relative symlink.
/// - If target resolves outside the template, creates an absolute symlink.
/// - If the target cannot be found (broken symlink), warns and preserves the original target.
fn copy_symlink(source_path: &Path, dest_path: &Path, source_dir: &Path, dest_dir: &Path) -> Result<()> {
    let raw_target = fs::read_link(source_path)
        .with_context(|| format!("failed to read symlink: {}", source_path.display()))?;

    let source_parent = source_path.parent().unwrap_or(source_dir);
    let absolute_target = if raw_target.is_absolute() {
        raw_target.clone()
    } else {
        source_parent.join(&raw_target)
    };

    let new_target: PathBuf = match absolute_target.canonicalize() {
        Ok(canonical_target) => {
            let canonical_source = source_dir
                .canonicalize()
                .unwrap_or_else(|_| source_dir.to_path_buf());
            if let Ok(target_rel) = canonical_target.strip_prefix(&canonical_source) {
                if raw_target.is_relative() {
                    // Same relative target works identically in the destination.
                    raw_target
                } else {
                    // Absolute target inside template: compute relative from dest symlink location.
                    let dest_parent = dest_path.parent().unwrap_or(dest_dir);
                    let target_in_dest = dest_dir.join(target_rel);
                    relative_path_between(dest_parent, &target_in_dest)
                }
            } else {
                // Target is outside the template: use the canonical absolute path.
                canonical_target
            }
        }
        Err(_) => {
            eprintln!(
                "warning: symlink '{}' points to '{}' which does not exist; creating anyway",
                source_path.display(),
                raw_target.display()
            );
            raw_target
        }
    };

    #[cfg(unix)]
    std::os::unix::fs::symlink(&new_target, dest_path)
        .with_context(|| format!("failed to create symlink: {}", dest_path.display()))?;

    #[cfg(not(unix))]
    {
        let _ = new_target;
        anyhow::bail!("symlinks are not supported on this platform");
    }

    Ok(())
}

/// Walks the source tree and returns the destination paths that already exist.
/// Used by `copy_template` to pre-flight a `NoOverwrite` copy before writing anything.
fn collect_collisions(source_dir: &Path, dest_dir: &Path, globset: &GlobSet) -> Result<Vec<PathBuf>> {
    let mut collisions = Vec::new();
    let walker = WalkDir::new(source_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.path() == source_dir || !should_skip_entry(entry, source_dir, globset)
        });
    for entry in walker {
        let entry = entry.with_context(|| "walkdir entry error")?;
        let path = entry.path();
        if path == source_dir || entry.file_type().is_dir() {
            continue;
        }
        let relative = path.strip_prefix(source_dir).with_context(|| "strip_prefix")?;
        let dest_path = dest_dir.join(relative);
        if dest_path.symlink_metadata().is_ok() {
            collisions.push(dest_path);
        }
    }
    Ok(collisions)
}

/// Copy template from `source_dir` to `dest_dir`.
/// `.git` is always excluded. `exclude` patterns are matched against each path
/// component and the full relative path. Symlinks are recreated. Preserves file permissions.
pub fn copy_template(
    source_dir: &Path,
    dest_dir: &Path,
    exclude: &[String],
    write_mode: &WriteMode,
) -> Result<()> {
    if !source_dir.is_dir() {
        anyhow::bail!("source is not a directory: {}", source_dir.display());
    }
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("failed to create destination: {}", dest_dir.display()))?;

    let globset = build_globset(exclude)?;

    if *write_mode == WriteMode::NoOverwrite {
        let collisions = collect_collisions(source_dir, dest_dir, &globset)?;
        if !collisions.is_empty() {
            return Err(TemplativeError::FilesWouldBeOverwritten { paths: collisions }.into());
        }
    }

    // `copy_mode` starts as `write_mode` and may be escalated to Overwrite or SkipOverwrite
    // for the rest of the session when the user picks an "apply to all" option.
    let mut copy_mode = write_mode.clone();

    let walker = WalkDir::new(source_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            let path = entry.path();
            if path == source_dir {
                return true;
            }
            !should_skip_entry(entry, source_dir, &globset)
        });

    for entry in walker {
        let entry = entry.with_context(|| "walkdir entry error")?;
        let path = entry.path();
        if path == source_dir {
            continue;
        }
        let relative = path
            .strip_prefix(source_dir)
            .with_context(|| "strip_prefix")?;
        let dest_path = dest_dir.join(relative);

        if path.is_symlink() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create parent: {}", parent.display()))?;
            }
            if dest_path.symlink_metadata().is_ok() {
                match copy_mode {
                    WriteMode::Strict | WriteMode::Overwrite | WriteMode::NoOverwrite => {
                        fs::remove_file(&dest_path)
                            .with_context(|| format!("failed to remove existing: {}", dest_path.display()))?;
                    }
                    WriteMode::SkipOverwrite => continue,
                    WriteMode::Ask => match prompt_file(&dest_path)? {
                        FileChoice::Overwrite => {
                            fs::remove_file(&dest_path).ok();
                        }
                        FileChoice::Skip => continue,
                        FileChoice::OverwriteAll => {
                            copy_mode = WriteMode::Overwrite;
                            fs::remove_file(&dest_path).ok();
                        }
                        FileChoice::SkipAll => {
                            copy_mode = WriteMode::SkipOverwrite;
                            continue;
                        }
                        FileChoice::Abort => anyhow::bail!("aborted by user"),
                    },
                }
            }
            copy_symlink(path, &dest_path, source_dir, dest_dir)?;
            continue;
        }

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)
                .with_context(|| format!("failed to create dir: {}", dest_path.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create parent: {}", parent.display()))?;
            }

            if dest_path.exists() {
                match copy_mode {
                    WriteMode::Strict | WriteMode::Overwrite | WriteMode::NoOverwrite => {}
                    WriteMode::SkipOverwrite => continue,
                    WriteMode::Ask => match prompt_file(&dest_path)? {
                        FileChoice::Overwrite => {}
                        FileChoice::Skip => continue,
                        FileChoice::OverwriteAll => {
                            copy_mode = WriteMode::Overwrite;
                        }
                        FileChoice::SkipAll => {
                            copy_mode = WriteMode::SkipOverwrite;
                            continue;
                        }
                        FileChoice::Abort => anyhow::bail!("aborted by user"),
                    },
                }
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

    fn default_exclude() -> Vec<String> {
        vec!["node_modules".into(), ".DS_Store".into()]
    }

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

        copy_template(&source, &dest, &default_exclude(), &WriteMode::Strict).unwrap();

        assert!(dest.join("src/main.rs").exists());
        assert!(dest.join("Cargo.toml").exists());
        assert!(dest.join("deep/nested/file.txt").exists());
        assert_eq!(fs::read_to_string(dest.join("deep/nested/file.txt")).unwrap(), "hello");
        assert!(!dest.join(".git").exists());
        assert!(!dest.join("node_modules").exists());
        assert!(!dest.join(".DS_Store").exists());
    }

    #[test]
    #[cfg(unix)]
    fn relative_symlink_inside_template_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();
        std::os::unix::fs::symlink("file.txt", source.join("link.txt")).unwrap();

        copy_template(&source, &dest, &[], &WriteMode::Strict).unwrap();

        assert!(dest.join("file.txt").exists());
        let link_target = fs::read_link(dest.join("link.txt")).unwrap();
        assert_eq!(link_target, Path::new("file.txt"));
        assert_eq!(fs::read_to_string(dest.join("link.txt")).unwrap(), "content");
    }

    #[test]
    #[cfg(unix)]
    fn broken_symlink_creates_with_original_target() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        std::os::unix::fs::symlink("nonexistent.txt", source.join("broken.txt")).unwrap();

        copy_template(&source, &dest, &[], &WriteMode::Strict).unwrap();

        let link_target = fs::read_link(dest.join("broken.txt")).unwrap();
        assert_eq!(link_target, Path::new("nonexistent.txt"));
        // Dest symlink exists but is broken (target doesn't exist).
        assert!(!dest.join("broken.txt").exists());
        assert!(dest.join("broken.txt").symlink_metadata().is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn symlink_outside_template_becomes_absolute() {
        let temp = tempfile::tempdir().unwrap();
        let external = temp.path().join("external.txt");
        fs::write(&external, "external content").unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        std::os::unix::fs::symlink(&external, source.join("link.txt")).unwrap();

        copy_template(&source, &dest, &[], &WriteMode::Strict).unwrap();

        let link_target = fs::read_link(dest.join("link.txt")).unwrap();
        assert!(link_target.is_absolute());
        assert_eq!(fs::read_to_string(dest.join("link.txt")).unwrap(), "external content");
    }

    #[test]
    fn glob_pattern_excludes_matching_files() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("main.rs"), "fn main() {}").unwrap();
        fs::write(source.join("debug.log"), "log content").unwrap();
        fs::write(source.join("error.log"), "error content").unwrap();

        copy_template(&source, &dest, &["*.log".into()], &WriteMode::Strict).unwrap();

        assert!(dest.join("main.rs").exists());
        assert!(!dest.join("debug.log").exists());
        assert!(!dest.join("error.log").exists());
    }

    #[test]
    fn glob_pattern_excludes_directory_tree() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(source.join("dist")).unwrap();
        fs::write(source.join("index.html"), "hello").unwrap();
        fs::write(source.join("dist/bundle.js"), "bundle").unwrap();

        copy_template(&source, &dest, &["dist".into()], &WriteMode::Strict).unwrap();

        assert!(dest.join("index.html").exists());
        assert!(!dest.join("dist").exists());
    }

    #[test]
    fn git_always_excluded_with_empty_exclude_list() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(source.join(".git")).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();
        fs::write(source.join(".git/config"), "[core]").unwrap();

        copy_template(&source, &dest, &[], &WriteMode::Strict).unwrap();

        assert!(dest.join("file.txt").exists());
        assert!(!dest.join(".git").exists());
    }

    #[test]
    fn no_overwrite_errors_on_existing_file() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(source.join("file.txt"), "new content").unwrap();
        fs::write(dest.join("file.txt"), "original content").unwrap();

        let result = copy_template(&source, &dest, &[], &WriteMode::NoOverwrite);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<TemplativeError>(),
            Some(TemplativeError::FilesWouldBeOverwritten { .. })
        ));
        assert_eq!(fs::read_to_string(dest.join("file.txt")).unwrap(), "original content");
    }

    #[test]
    fn no_overwrite_writes_nothing_when_collision_exists() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(source.join("new.txt"), "new content").unwrap();
        fs::write(source.join("collision.txt"), "new content").unwrap();
        fs::write(dest.join("collision.txt"), "original").unwrap();

        let result = copy_template(&source, &dest, &[], &WriteMode::NoOverwrite);

        assert!(result.is_err());
        // new.txt must not have been written — error was raised before any writes
        assert!(!dest.join("new.txt").exists());
    }

    #[test]
    fn skip_overwrite_preserves_existing_file() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(source.join("existing.txt"), "new content").unwrap();
        fs::write(source.join("new.txt"), "brand new").unwrap();
        fs::write(dest.join("existing.txt"), "original content").unwrap();

        copy_template(&source, &dest, &[], &WriteMode::SkipOverwrite).unwrap();

        assert_eq!(fs::read_to_string(dest.join("existing.txt")).unwrap(), "original content");
        assert_eq!(fs::read_to_string(dest.join("new.txt")).unwrap(), "brand new");
    }

    #[test]
    fn overwrite_replaces_existing_file() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(source.join("file.txt"), "new content").unwrap();
        fs::write(dest.join("file.txt"), "original content").unwrap();

        copy_template(&source, &dest, &[], &WriteMode::Overwrite).unwrap();

        assert_eq!(fs::read_to_string(dest.join("file.txt")).unwrap(), "new content");
    }

    #[test]
    #[cfg(unix)]
    fn no_overwrite_errors_on_existing_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();
        std::os::unix::fs::symlink("file.txt", source.join("link.txt")).unwrap();
        std::os::unix::fs::symlink("file.txt", dest.join("link.txt")).unwrap();

        let result = copy_template(&source, &dest, &[], &WriteMode::NoOverwrite);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<TemplativeError>(),
            Some(TemplativeError::FilesWouldBeOverwritten { .. })
        ));
    }

    #[test]
    #[cfg(unix)]
    fn skip_overwrite_preserves_existing_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("template");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();
        std::os::unix::fs::symlink("file.txt", source.join("link.txt")).unwrap();
        // Existing symlink points elsewhere
        std::os::unix::fs::symlink("other.txt", dest.join("link.txt")).unwrap();

        copy_template(&source, &dest, &[], &WriteMode::SkipOverwrite).unwrap();

        assert_eq!(fs::read_link(dest.join("link.txt")).unwrap(), Path::new("other.txt"));
    }
}

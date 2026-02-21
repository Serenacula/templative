use std::sync::Mutex;

use tempfile::tempdir;

use super::*;

// Serialise all tests that touch TEMPLATIVE_CONFIG_DIR
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct IsolatedConfig {
    _guard: std::sync::MutexGuard<'static, ()>,
    dir: tempfile::TempDir,
}

impl IsolatedConfig {
    fn new() -> Self {
        let guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        std::env::set_var("TEMPLATIVE_CONFIG_DIR", dir.path());
        Self { _guard: guard, dir }
    }

    fn path(&self) -> &std::path::Path {
        self.dir.path()
    }
}

impl Drop for IsolatedConfig {
    fn drop(&mut self) {
        std::env::remove_var("TEMPLATIVE_CONFIG_DIR");
    }
}

#[test]
fn cmd_add_registers_local_template() {
    let _config = IsolatedConfig::new();
    let template_dir = tempdir().unwrap();

    cmd_add(
        template_dir.path().to_str().unwrap().to_string(),
        Some("my-template".into()),
        None,
        None,
        None,
        vec![],
        None,
    )
    .unwrap();

    let registry = crate::registry::Registry::load().unwrap();
    assert!(registry.get("my-template").is_some());
}

#[test]
fn cmd_add_duplicate_name_errors() {
    let _config = IsolatedConfig::new();
    let template_dir = tempdir().unwrap();
    let path = template_dir.path().to_str().unwrap().to_string();

    cmd_add(path.clone(), Some("dup".into()), None, None, None, vec![], None).unwrap();
    let result = cmd_add(path, Some("dup".into()), None, None, None, vec![], None);
    assert!(result.is_err());
}

#[test]
fn cmd_remove_deregisters_template() {
    let _config = IsolatedConfig::new();
    let template_dir = tempdir().unwrap();

    cmd_add(
        template_dir.path().to_str().unwrap().to_string(),
        Some("to-remove".into()),
        None,
        None,
        None,
        vec![],
        None,
    )
    .unwrap();

    cmd_remove(vec!["to-remove".into()]).unwrap();

    let registry = crate::registry::Registry::load().unwrap();
    assert!(registry.get("to-remove").is_none());
}

#[test]
fn cmd_remove_nonexistent_errors() {
    let _config = IsolatedConfig::new();
    let result = cmd_remove(vec!["ghost".into()]);
    assert!(result.is_err());
}

#[test]
fn cmd_remove_multiple_all_or_nothing() {
    let _config = IsolatedConfig::new();
    let template_dir = tempdir().unwrap();

    cmd_add(
        template_dir.path().to_str().unwrap().to_string(),
        Some("real".into()),
        None,
        None,
        None,
        vec![],
        None,
    )
    .unwrap();

    // "ghost" doesn't exist — neither should be removed
    let result = cmd_remove(vec!["real".into(), "ghost".into()]);
    assert!(result.is_err());

    let registry = crate::registry::Registry::load().unwrap();
    assert!(registry.get("real").is_some());
}

#[test]
fn cmd_list_succeeds_with_empty_registry() {
    let _config = IsolatedConfig::new();
    cmd_list(false, false).unwrap();
}

#[test]
fn cmd_list_succeeds_with_templates() {
    let _config = IsolatedConfig::new();
    let template_dir = tempdir().unwrap();

    cmd_add(
        template_dir.path().to_str().unwrap().to_string(),
        Some("listed".into()),
        Some("a template".into()),
        None,
        None,
        vec![],
        None,
    )
    .unwrap();

    cmd_list(false, false).unwrap();
}

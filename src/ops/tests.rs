use std::path::PathBuf;

use tempfile::tempdir;

use crate::errors::TemplativeError;
use crate::registry::{Registry, Template};
use crate::test_env::ENV_LOCK;

use super::*;

struct IsolatedConfig {
    _guard: std::sync::MutexGuard<'static, ()>,
    dir: tempfile::TempDir,
}

impl IsolatedConfig {
    fn new() -> Self {
        let guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        unsafe { std::env::set_var("TEMPLATIVE_CONFIG_DIR", dir.path()); }
        Self { _guard: guard, dir }
    }

    fn path(&self) -> &std::path::Path {
        self.dir.path()
    }
}

impl Drop for IsolatedConfig {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("TEMPLATIVE_CONFIG_DIR"); }
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

fn setup_registry(config: &IsolatedConfig, templates: Vec<Template>) {
    let mut registry = Registry::new();
    for template in templates {
        registry.templates.push(template);
    }
    registry.save_to_path(&config.path().join("templates.json")).unwrap();
}

fn empty_change_options() -> ChangeOptions {
    ChangeOptions {
        name: None,
        description: None,
        location: None,
        git: None,
        pre_init: None,
        post_init: None,
        git_ref: None,
        exclude: None,
        write_mode: None,
    }
}

#[test]
fn cmd_change_errors_when_template_not_found() {
    let config = IsolatedConfig::new();
    setup_registry(&config, vec![]);
    let result = cmd_change("nonexistent".into(), ChangeOptions { name: Some("x".into()), ..empty_change_options() });
    assert!(matches!(
        result.unwrap_err().downcast_ref::<TemplativeError>(),
        Some(TemplativeError::TemplateNotFound { .. })
    ));
}

#[test]
fn cmd_change_errors_when_new_name_already_exists() {
    let config = IsolatedConfig::new();
    setup_registry(&config, vec![make_template("foo", "/tmp"), make_template("bar", "/tmp")]);
    let result = cmd_change("foo".into(), ChangeOptions { name: Some("bar".into()), ..empty_change_options() });
    assert!(matches!(
        result.unwrap_err().downcast_ref::<TemplativeError>(),
        Some(TemplativeError::TemplateExists { .. })
    ));
}

#[test]
fn cmd_change_updates_name_successfully() {
    let config = IsolatedConfig::new();
    setup_registry(&config, vec![make_template("foo", "/tmp")]);
    let result = cmd_change("foo".into(), ChangeOptions { name: Some("bar".into()), ..empty_change_options() });
    let registry = Registry::load_from_path(&config.path().join("templates.json")).unwrap();
    assert!(result.is_ok());
    assert!(registry.get("bar").is_some());
    assert!(registry.get("foo").is_none());
}

#[test]
fn cmd_change_rejects_nonexistent_location() {
    let config = IsolatedConfig::new();
    setup_registry(&config, vec![make_template("foo", "/tmp")]);
    let result = cmd_change("foo".into(), ChangeOptions {
        location: Some(PathBuf::from("/this/path/does/not/exist/ever")),
        ..empty_change_options()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("path not found"));
}

#[test]
fn cmd_change_updates_location_to_existing_path() {
    let config = IsolatedConfig::new();
    let new_location = tempdir().unwrap();
    setup_registry(&config, vec![make_template("foo", "/tmp")]);
    let result = cmd_change("foo".into(), ChangeOptions {
        location: Some(new_location.path().to_path_buf()),
        ..empty_change_options()
    });
    let registry = Registry::load_from_path(&config.path().join("templates.json")).unwrap();
    assert!(result.is_ok());
    let expected = new_location.path().canonicalize().unwrap().to_string_lossy().into_owned();
    assert_eq!(registry.get("foo").unwrap().location, expected);
}

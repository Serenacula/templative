use crate::config::{Config, GitMode, UpdateOnInit};
use crate::registry::Template;

/// Merged settings for a single `init` invocation.
/// Resolution order: CLI flag > template field > config default.
#[derive(Debug)]
pub struct ResolvedOptions {
    pub git: GitMode,
    #[allow(dead_code)]
    pub commit: Option<String>,
    pub pre_init: Option<String>,
    pub post_init: Option<String>,
    pub no_cache: bool,
    pub git_ref: Option<String>,
    pub update_on_init: UpdateOnInit,
}

impl ResolvedOptions {
    pub fn build(config: &Config, template: &Template, git_flag: Option<GitMode>) -> Self {
        Self {
            git: git_flag.or_else(|| template.git.clone()).unwrap_or_else(|| config.git.clone()),
            commit: template.commit.clone(),
            pre_init: template.pre_init.clone(),
            post_init: template.post_init.clone(),
            no_cache: template.no_cache.unwrap_or(config.no_cache),
            git_ref: template.git_ref.clone(),
            update_on_init: config.update_on_init.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(git: GitMode) -> Config {
        Config { version: 1, git, update_on_init: UpdateOnInit::OnlyUrl, no_cache: false }
    }

    fn make_template(git: Option<GitMode>) -> Template {
        Template {
            name: "test".into(),
            location: "/tmp".into(),
            git,
            description: None,
            commit: None,
            pre_init: None,
            post_init: None,
            git_ref: None,
            no_cache: None,
        }
    }

    #[test]
    fn flag_overrides_template_and_config() {
        let resolved = ResolvedOptions::build(
            &make_config(GitMode::Fresh),
            &make_template(Some(GitMode::Fresh)),
            Some(GitMode::NoGit),
        );
        assert_eq!(resolved.git, GitMode::NoGit);
    }

    #[test]
    fn template_overrides_config() {
        let resolved = ResolvedOptions::build(
            &make_config(GitMode::Fresh),
            &make_template(Some(GitMode::Preserve)),
            None,
        );
        assert_eq!(resolved.git, GitMode::Preserve);
    }

    #[test]
    fn config_used_when_no_flag_or_template() {
        let resolved = ResolvedOptions::build(
            &make_config(GitMode::NoGit),
            &make_template(None),
            None,
        );
        assert_eq!(resolved.git, GitMode::NoGit);
    }

    #[test]
    fn defaults_to_fresh_with_default_config() {
        let resolved = ResolvedOptions::build(
            &make_config(GitMode::Fresh),
            &make_template(None),
            None,
        );
        assert_eq!(resolved.git, GitMode::Fresh);
    }

    #[test]
    fn no_cache_resolves_from_template() {
        let mut template = make_template(None);
        template.no_cache = Some(true);
        let resolved = ResolvedOptions::build(&make_config(GitMode::Fresh), &template, None);
        assert!(resolved.no_cache);
    }

    #[test]
    fn no_cache_resolves_from_config() {
        let mut config = make_config(GitMode::Fresh);
        config.no_cache = true;
        let resolved = ResolvedOptions::build(&config, &make_template(None), None);
        assert!(resolved.no_cache);
    }

    #[test]
    fn template_no_cache_overrides_config() {
        let mut config = make_config(GitMode::Fresh);
        config.no_cache = true;
        let mut template = make_template(None);
        template.no_cache = Some(false);
        let resolved = ResolvedOptions::build(&config, &template, None);
        assert!(!resolved.no_cache);
    }

    #[test]
    fn git_ref_resolves_from_template() {
        let mut template = make_template(None);
        template.git_ref = Some("v1.0".into());
        let resolved = ResolvedOptions::build(&make_config(GitMode::Fresh), &template, None);
        assert_eq!(resolved.git_ref.as_deref(), Some("v1.0"));
    }

    #[test]
    fn update_on_init_comes_from_config() {
        let mut config = make_config(GitMode::Fresh);
        config.update_on_init = UpdateOnInit::Never;
        let resolved = ResolvedOptions::build(&config, &make_template(None), None);
        assert_eq!(resolved.update_on_init, UpdateOnInit::Never);
    }
}

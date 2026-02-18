use crate::config::{Config, UpdateOnInit};
use crate::registry::Template;

/// Merged settings for a single `init` invocation.
/// Resolution order: CLI flag > template field > config default.
#[derive(Debug)]
pub struct ResolvedOptions {
    pub git: bool,
    #[allow(dead_code)]
    pub commit: Option<String>,
    pub pre_init: Option<String>,
    pub post_init: Option<String>,
    pub fresh: bool,
    pub no_cache: bool,
    pub git_ref: Option<String>,
    pub update_on_init: UpdateOnInit,
}

impl ResolvedOptions {
    pub fn build(config: &Config, template: &Template, git_flag: Option<bool>, fresh_flag: Option<bool>) -> Self {
        Self {
            git: git_flag.or(template.git).unwrap_or(config.git),
            commit: template.commit.clone(),
            pre_init: template.pre_init.clone(),
            post_init: template.post_init.clone(),
            fresh: fresh_flag.or(template.fresh).unwrap_or(config.fresh),
            no_cache: template.no_cache.unwrap_or(config.no_cache),
            git_ref: template.git_ref.clone(),
            update_on_init: config.update_on_init.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(git: bool) -> Config {
        Config { version: 1, git, fresh: true, update_on_init: UpdateOnInit::OnlyUrl, no_cache: false }
    }

    fn make_template(git: Option<bool>) -> Template {
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
            fresh: None,
        }
    }

    #[test]
    fn flag_overrides_template_and_config() {
        let resolved = ResolvedOptions::build(&make_config(true), &make_template(Some(true)), Some(false), None);
        assert!(!resolved.git);
    }

    #[test]
    fn template_overrides_config() {
        let resolved = ResolvedOptions::build(&make_config(true), &make_template(Some(false)), None, None);
        assert!(!resolved.git);
    }

    #[test]
    fn config_used_when_no_flag_or_template() {
        let resolved = ResolvedOptions::build(&make_config(false), &make_template(None), None, None);
        assert!(!resolved.git);
    }

    #[test]
    fn defaults_to_true_with_default_config() {
        let resolved = ResolvedOptions::build(&make_config(true), &make_template(None), None, None);
        assert!(resolved.git);
    }

    #[test]
    fn fresh_flag_overrides_template_and_config() {
        let resolved = ResolvedOptions::build(&make_config(true), &make_template(None), None, Some(false));
        assert!(!resolved.fresh);
    }

    #[test]
    fn fresh_template_overrides_config() {
        let mut t = make_template(None);
        t.fresh = Some(false);
        let resolved = ResolvedOptions::build(&make_config(true), &t, None, None);
        assert!(!resolved.fresh);
    }

    #[test]
    fn no_cache_resolves_from_template() {
        let mut t = make_template(None);
        t.no_cache = Some(true);
        let resolved = ResolvedOptions::build(&make_config(true), &t, None, None);
        assert!(resolved.no_cache);
    }

    #[test]
    fn no_cache_resolves_from_config() {
        let mut config = make_config(true);
        config.no_cache = true;
        let resolved = ResolvedOptions::build(&config, &make_template(None), None, None);
        assert!(resolved.no_cache);
    }

    #[test]
    fn template_no_cache_overrides_config() {
        let mut config = make_config(true);
        config.no_cache = true;
        let mut t = make_template(None);
        t.no_cache = Some(false);
        let resolved = ResolvedOptions::build(&config, &t, None, None);
        assert!(!resolved.no_cache);
    }

    #[test]
    fn git_ref_resolves_from_template() {
        let mut t = make_template(None);
        t.git_ref = Some("v1.0".into());
        let resolved = ResolvedOptions::build(&make_config(true), &t, None, None);
        assert_eq!(resolved.git_ref.as_deref(), Some("v1.0"));
    }

    #[test]
    fn update_on_init_comes_from_config() {
        let mut config = make_config(true);
        config.update_on_init = UpdateOnInit::Never;
        let resolved = ResolvedOptions::build(&config, &make_template(None), None, None);
        assert_eq!(resolved.update_on_init, UpdateOnInit::Never);
    }
}

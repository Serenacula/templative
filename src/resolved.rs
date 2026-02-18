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
}

use crate::config::Config;
use crate::registry::Template;

/// Merged settings for a single `init` invocation.
/// Resolution order: CLI flag > template field > config default.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ResolvedOptions {
    pub git: bool,
    pub commit: Option<String>,
    pub pre_init: Option<String>,
    pub post_init: Option<String>,
}

impl ResolvedOptions {
    pub fn build(config: &Config, template: &Template, git_flag: Option<bool>) -> Self {
        Self {
            git: git_flag.or(template.git).unwrap_or(config.git),
            commit: template.commit.clone(),
            pre_init: template.pre_init.clone(),
            post_init: template.post_init.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(git: bool) -> Config {
        Config { version: 1, git }
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
        }
    }

    #[test]
    fn flag_overrides_template_and_config() {
        let resolved = ResolvedOptions::build(&make_config(true), &make_template(Some(true)), Some(false));
        assert!(!resolved.git);
    }

    #[test]
    fn template_overrides_config() {
        let resolved = ResolvedOptions::build(&make_config(true), &make_template(Some(false)), None);
        assert!(!resolved.git);
    }

    #[test]
    fn config_used_when_no_flag_or_template() {
        let resolved = ResolvedOptions::build(&make_config(false), &make_template(None), None);
        assert!(!resolved.git);
    }

    #[test]
    fn defaults_to_true_with_default_config() {
        let resolved = ResolvedOptions::build(&make_config(true), &make_template(None), None);
        assert!(resolved.git);
    }
}

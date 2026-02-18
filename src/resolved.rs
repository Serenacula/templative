use crate::config::Config;
use crate::registry::Template;

/// Merged settings for a single `init` invocation.
/// Resolution order: CLI flag > template field > config default.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ResolvedOptions {
    pub commit: Option<String>,
    pub pre_init: Option<String>,
    pub post_init: Option<String>,
}

impl ResolvedOptions {
    pub fn build(_config: &Config, template: &Template) -> Self {
        Self {
            commit: template.commit.clone(),
            pre_init: template.pre_init.clone(),
            post_init: template.post_init.clone(),
        }
    }
}

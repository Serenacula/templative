use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum TemplativeError {
    #[error("refusing to operate on {path:?}")]
    DangerousPath { path: PathBuf },

    #[error("target directory is not empty")]
    TargetNotEmpty,

    #[error("template not found: {name}")]
    TemplateNotFound { name: String },

    #[error("template name already exists: {name}")]
    TemplateExists { name: String },

    #[error("template path missing or unreadable: {path:?}")]
    TemplatePathMissing { path: PathBuf },

    #[error("unsupported registry version {found} (expected {expected}); delete {path} to start fresh")]
    UnsupportedRegistryVersion {
        found: u32,
        expected: u32,
        path: String,
    },

    #[error("unsupported config version (expected 1)")]
    UnsupportedConfigVersion,

    #[error("file would be overwritten: {path:?}")]
    FileWouldBeOverwritten { path: PathBuf },
}

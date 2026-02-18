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

    #[error("symlinks not supported yet")]
    SymlinkNotSupported,

    #[error("unsupported registry version (expected 1)")]
    UnsupportedRegistryVersion,

    #[error("unsupported config version (expected 1)")]
    UnsupportedConfigVersion,

}

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("could not determine a home directory (no HOME/USERPROFILE set)")]
    NoHomeDir,

    #[error("failed to create plugin directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("plugin source {0} does not exist")]
    SourceNotFound(PathBuf),

    #[error("failed to install plugin to {path}: {source}")]
    Install {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to launch plugin '{name}': {source}")]
    Launch {
        name: String,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, PluginError>;

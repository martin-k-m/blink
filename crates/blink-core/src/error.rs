use std::path::PathBuf;

/// Errors produced by Blink's core detection and configuration systems.
#[derive(Debug, thiserror::Error)]
pub enum BlinkError {
    #[error("path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse configuration at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("failed to serialize configuration: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("failed to parse manifest {path}: {source}")]
    ManifestParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("no recognizable project found in {0}")]
    UnknownProject(PathBuf),
}

pub type Result<T> = std::result::Result<T, BlinkError>;

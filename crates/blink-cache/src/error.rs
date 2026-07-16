use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("failed to read cache file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write cache file {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse cache file {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to serialize cache: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error(
        "could not determine a platform cache directory (no HOME/USERPROFILE/LOCALAPPDATA set)"
    )]
    NoCacheDir,
}

pub type Result<T> = std::result::Result<T, CacheError>;

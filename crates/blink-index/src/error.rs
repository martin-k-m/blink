use std::path::PathBuf;

use thiserror::Error;

/// Errors raised while building, saving, or loading an index.
#[derive(Debug, Error)]
pub enum IndexError {
    #[error("i/o error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to serialize index: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, IndexError>;

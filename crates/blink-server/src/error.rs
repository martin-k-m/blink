use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("failed to watch {path}: {source}")]
    Watch {
        path: PathBuf,
        #[source]
        source: notify::Error,
    },

    #[error("failed to bind dev server to {addr}: {source}")]
    Bind {
        addr: String,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, ServerError>;

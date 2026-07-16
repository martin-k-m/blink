/// Errors from parsing a manifest whose syntax is malformed. Manifests that
/// are merely *empty* of the fields Blink cares about are not errors here —
/// callers get a manifest with empty dependency lists instead, matching how
/// npm/cargo themselves tolerate sparse manifests.
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ParserError>;

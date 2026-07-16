use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single file's fingerprint as of the last scan: its size and content
/// hash. Two entries are considered equal, and therefore "unchanged",
/// when both match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub size: u64,
    pub hash: String,
}

impl CacheEntry {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let hash = hex::encode(hasher.finalize());
        Self {
            size: bytes.len() as u64,
            hash,
        }
    }
}

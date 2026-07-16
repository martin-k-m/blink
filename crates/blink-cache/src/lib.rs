//! Content-hash based build cache for Blink: fingerprints project files so
//! `blink build` can skip work that hasn't changed since the last run.

mod entry;
mod error;
mod store;

#[cfg(test)]
mod tests;

pub use entry::CacheEntry;
pub use error::{CacheError, Result};
pub use store::{Cache, CacheDiff, CACHE_DIR};

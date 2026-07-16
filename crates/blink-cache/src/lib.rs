//! Content-hash based caching for Blink: a project-local build cache
//! (`.blink/cache.json`) so `blink build` can skip work that hasn't
//! changed, plus a global per-user cache (in the platform cache
//! directory) so analysis results can be reused across runs.

mod analysis_cache;
mod entry;
mod error;
mod global;
mod store;

#[cfg(test)]
mod tests;

pub use analysis_cache::AnalysisCache;
pub use entry::CacheEntry;
pub use error::{CacheError, Result};
pub use global::global_cache_dir;
pub use store::{Cache, CacheDiff, CACHE_DIR};

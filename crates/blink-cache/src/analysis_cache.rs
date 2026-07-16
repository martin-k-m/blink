use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{CacheError, Result};
use crate::global::global_cache_dir;
use crate::store::Cache;

/// Caches an arbitrary serializable value (typically an analysis result)
/// keyed by project path, in Blink's global per-user cache directory. Each
/// entry is validated against a [`Cache`] file-hash snapshot: if the
/// project's files haven't changed since the value was cached, the stored
/// value is reused instead of recomputing it.
pub struct AnalysisCache {
    dir: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct CachedEntry<T> {
    snapshot: Cache,
    value: T,
}

impl AnalysisCache {
    /// Open the cache in Blink's platform-appropriate global cache directory.
    pub fn open() -> Result<Self> {
        Ok(Self {
            dir: global_cache_dir()?,
        })
    }

    /// Open a cache rooted at an arbitrary directory. Used by tests to
    /// avoid touching the real platform cache directory.
    pub fn open_at(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Look up a cached value for `project_root`, valid only if `snapshot`
    /// (a fresh [`Cache::scan`] of the project) matches what was stored.
    /// Returns `None` on a miss, a mismatch, or any I/O/parse failure —
    /// callers should treat that identically to "nothing cached yet" and
    /// fall back to recomputing.
    pub fn get<T: DeserializeOwned>(&self, project_root: &Path, snapshot: &Cache) -> Option<T> {
        let path = self.entry_path(project_root);
        let raw = std::fs::read_to_string(path).ok()?;
        let cached: CachedEntry<T> = serde_json::from_str(&raw).ok()?;
        (cached.snapshot == *snapshot).then_some(cached.value)
    }

    /// Store `value` for `project_root`, associated with `snapshot` so a
    /// future [`Self::get`] can tell whether it's still valid.
    pub fn set<T: Serialize>(&self, project_root: &Path, snapshot: Cache, value: &T) -> Result<()> {
        let path = self.entry_path(project_root);
        let entry = CachedEntry { snapshot, value };
        let contents = serde_json::to_string(&entry)?;
        std::fs::write(&path, contents).map_err(|source| CacheError::Write { path, source })
    }

    fn entry_path(&self, project_root: &Path) -> PathBuf {
        self.dir
            .join(format!("{}.json", project_cache_key(project_root)))
    }
}

/// A stable, filesystem-safe key derived from a project's canonical path.
fn project_cache_key(root: &Path) -> String {
    let absolute = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(absolute.to_string_lossy().as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::AnalysisCache;
    use crate::store::Cache;

    #[test]
    fn miss_when_nothing_cached() {
        let cache_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        let cache = AnalysisCache::open_at(cache_dir.path().to_path_buf());
        let snapshot = Cache::scan(project_dir.path());

        let value: Option<String> = cache.get(project_dir.path(), &snapshot);

        assert!(value.is_none());
    }

    #[test]
    fn hit_when_snapshot_matches() {
        let cache_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        std::fs::write(project_dir.path().join("a.txt"), "hello").unwrap();
        let cache = AnalysisCache::open_at(cache_dir.path().to_path_buf());
        let snapshot = Cache::scan(project_dir.path());

        cache
            .set(
                project_dir.path(),
                snapshot.clone(),
                &"cached value".to_string(),
            )
            .unwrap();
        let value: Option<String> = cache.get(project_dir.path(), &snapshot);

        assert_eq!(value.as_deref(), Some("cached value"));
    }

    #[test]
    fn miss_when_files_changed_since_caching() {
        let cache_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        std::fs::write(project_dir.path().join("a.txt"), "hello").unwrap();
        let cache = AnalysisCache::open_at(cache_dir.path().to_path_buf());
        let stale_snapshot = Cache::scan(project_dir.path());
        cache
            .set(
                project_dir.path(),
                stale_snapshot,
                &"cached value".to_string(),
            )
            .unwrap();

        std::fs::write(project_dir.path().join("a.txt"), "goodbye").unwrap();
        let fresh_snapshot = Cache::scan(project_dir.path());
        let value: Option<String> = cache.get(project_dir.path(), &fresh_snapshot);

        assert!(value.is_none());
    }

    #[test]
    fn different_projects_get_different_cache_entries() {
        let cache_dir = TempDir::new().unwrap();
        let project_a = TempDir::new().unwrap();
        let project_b = TempDir::new().unwrap();
        let cache = AnalysisCache::open_at(cache_dir.path().to_path_buf());
        let snapshot_a = Cache::scan(project_a.path());
        let snapshot_b = Cache::scan(project_b.path());

        cache
            .set(project_a.path(), snapshot_a.clone(), &"value-a".to_string())
            .unwrap();

        let value_a: Option<String> = cache.get(project_a.path(), &snapshot_a);
        let value_b: Option<String> = cache.get(project_b.path(), &snapshot_b);

        assert_eq!(value_a.as_deref(), Some("value-a"));
        assert!(value_b.is_none());
    }
}

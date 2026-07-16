use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::entry::CacheEntry;
use crate::error::{CacheError, Result};

const IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".cache",
    ".blink",
];

/// Where Blink persists its build cache within a project, relative to the
/// project root.
pub const CACHE_DIR: &str = ".blink";
const CACHE_FILE: &str = "cache.json";

/// A snapshot of every tracked file's fingerprint, keyed by path relative
/// to the project root.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cache {
    entries: BTreeMap<String, CacheEntry>,
}

/// The result of comparing a freshly-scanned [`Cache`] against a
/// previously persisted one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheDiff {
    pub total: usize,
    pub unchanged: usize,
    pub changed: usize,
    pub added: usize,
    pub removed: usize,
}

impl Cache {
    /// Walk `root` and hash every tracked file to build a fresh snapshot.
    pub fn scan(root: &Path) -> Self {
        let paths: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .filter_entry(|entry| {
                if entry.file_type().is_dir() {
                    let name = entry.file_name().to_string_lossy();
                    !IGNORED_DIRS.contains(&name.as_ref())
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();

        let entries: BTreeMap<String, CacheEntry> = paths
            .par_iter()
            .filter_map(|path| {
                let bytes = std::fs::read(path).ok()?;
                let relative = path.strip_prefix(root).unwrap_or(path);
                Some((
                    relative.to_string_lossy().replace('\\', "/"),
                    CacheEntry::from_bytes(&bytes),
                ))
            })
            .collect();

        Self { entries }
    }

    pub fn file_count(&self) -> usize {
        self.entries.len()
    }

    /// Compare this (fresh) snapshot against `previous` (the last persisted
    /// snapshot) and summarize what changed.
    pub fn diff(&self, previous: &Cache) -> CacheDiff {
        let mut diff = CacheDiff {
            total: self.entries.len(),
            ..Default::default()
        };

        for (path, entry) in &self.entries {
            match previous.entries.get(path) {
                Some(prev_entry) if prev_entry == entry => diff.unchanged += 1,
                Some(_) => diff.changed += 1,
                None => diff.added += 1,
            }
        }
        diff.removed = previous
            .entries
            .keys()
            .filter(|path| !self.entries.contains_key(*path))
            .count();

        diff
    }

    fn cache_file_path(root: &Path) -> PathBuf {
        root.join(CACHE_DIR).join(CACHE_FILE)
    }

    /// Load a previously persisted cache for `root`, if one exists.
    pub fn load(root: &Path) -> Result<Option<Cache>> {
        let path = Self::cache_file_path(root);
        if !path.is_file() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(&path).map_err(|source| CacheError::Read {
            path: path.clone(),
            source,
        })?;
        let cache =
            serde_json::from_str(&raw).map_err(|source| CacheError::Parse { path, source })?;
        Ok(Some(cache))
    }

    /// Persist this cache snapshot to `root`/.blink/cache.json.
    pub fn save(&self, root: &Path) -> Result<()> {
        let dir = root.join(CACHE_DIR);
        std::fs::create_dir_all(&dir).map_err(|source| CacheError::Write {
            path: dir.clone(),
            source,
        })?;
        let path = Self::cache_file_path(root);
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, contents).map_err(|source| CacheError::Write { path, source })
    }
}

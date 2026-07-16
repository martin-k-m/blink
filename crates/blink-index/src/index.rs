use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use blink_core::effective_ignored_dirs;

use crate::error::{IndexError, Result};
use crate::record::{normalize_rel, FileLang, FileRecord, Symbol};

/// On-disk format version. Bumped when [`FileRecord`]/[`Index`] change shape so
/// a stale index is rebuilt from scratch instead of misparsed.
pub const INDEX_VERSION: u32 = 1;

/// Directory (under the project root) the index lives in — shared with the
/// build cache, since both are project-local, disposable, and git-ignored.
pub const INDEX_DIR: &str = ".blink";
pub const INDEX_FILE: &str = "index.json";

/// What a refresh changed, for reporting to the user. Every count is measured
/// against the files actually walked and the prior index — never estimated.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RefreshStats {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
    pub unchanged: usize,
}

impl RefreshStats {
    /// Whether the refresh actually rebuilt any file records.
    pub fn changed(&self) -> bool {
        self.added > 0 || self.updated > 0 || self.removed > 0
    }

    /// Files (re)hashed this refresh — the work the incremental path avoided
    /// for the `unchanged` ones.
    pub fn reprocessed(&self) -> usize {
        self.added + self.updated
    }
}

/// A project's file/symbol index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub version: u32,
    /// Whole seconds since the Unix epoch of the last refresh.
    pub generated_at: u64,
    /// Records keyed by `/`-separated path relative to the root.
    pub files: BTreeMap<String, FileRecord>,
    /// The project root. Not serialized — restored from the load location so
    /// an index file copied elsewhere still resolves paths correctly.
    #[serde(skip)]
    pub root: PathBuf,
}

impl Index {
    fn empty(root: &Path) -> Self {
        Self {
            version: INDEX_VERSION,
            generated_at: 0,
            files: BTreeMap::new(),
            root: root.to_path_buf(),
        }
    }

    /// Path of the index file for `root`.
    pub fn path_for(root: &Path) -> PathBuf {
        root.join(INDEX_DIR).join(INDEX_FILE)
    }

    /// Load an existing index for `root`, or `None` if it's missing, corrupt,
    /// or written by a different [`INDEX_VERSION`] (treated as "no index").
    pub fn load(root: &Path) -> Option<Self> {
        let raw = std::fs::read_to_string(Self::path_for(root)).ok()?;
        let mut index: Index = serde_json::from_str(&raw).ok()?;
        if index.version != INDEX_VERSION {
            return None;
        }
        index.root = root.to_path_buf();
        Some(index)
    }

    /// Persist this index to `root`/.blink/index.json.
    pub fn save(&self) -> Result<()> {
        let dir = self.root.join(INDEX_DIR);
        std::fs::create_dir_all(&dir).map_err(|source| IndexError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = Self::path_for(&self.root);
        let json = serde_json::to_string(self)?;
        std::fs::write(&path, json).map_err(|source| IndexError::Io { path, source })
    }

    /// Build a fresh index for `root` from scratch (ignores any existing one).
    pub fn build(root: &Path) -> Result<(Self, RefreshStats)> {
        Self::empty(root).refreshed(root)
    }

    /// Load the existing index for `root` (or start empty) and incrementally
    /// refresh it: only files whose size *or* mtime changed are re-hashed and
    /// re-parsed; unchanged files reuse their prior record.
    pub fn refresh(root: &Path) -> Result<(Self, RefreshStats)> {
        let prior = Self::load(root).unwrap_or_else(|| Self::empty(root));
        prior.refreshed(root)
    }

    fn refreshed(self, root: &Path) -> Result<(Self, RefreshStats)> {
        let ignored = effective_ignored_dirs(root);
        let current = walk_files(root, &ignored);

        let mut stats = RefreshStats::default();
        let mut to_build: Vec<String> = Vec::new();
        let mut files: BTreeMap<String, FileRecord> = BTreeMap::new();

        for (rel, size, mtime) in &current {
            match self.files.get(rel) {
                Some(prev) if prev.size == *size && prev.mtime == *mtime => {
                    files.insert(rel.clone(), prev.clone());
                    stats.unchanged += 1;
                }
                Some(_) => {
                    stats.updated += 1;
                    to_build.push(rel.clone());
                }
                None => {
                    stats.added += 1;
                    to_build.push(rel.clone());
                }
            }
        }

        // Anything in the prior index no longer present on disk was removed.
        stats.removed = self
            .files
            .keys()
            .filter(|p| !current.iter().any(|(rel, _, _)| rel == *p))
            .count();

        // Re-hash/parse the changed files in parallel; this is the expensive
        // part the unchanged-file fast path exists to avoid.
        let built: Vec<(String, FileRecord)> = to_build
            .par_iter()
            .filter_map(|rel| {
                FileRecord::build(root, rel)
                    .ok()
                    .map(|rec| (rel.clone(), rec))
            })
            .collect();
        for (rel, rec) in built {
            files.insert(rel, rec);
        }

        let index = Index {
            version: INDEX_VERSION,
            generated_at: now_secs(),
            files,
            root: root.to_path_buf(),
        };
        Ok((index, stats))
    }

    // --- Queries ------------------------------------------------------------

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn total_size(&self) -> u64 {
        self.files.values().map(|f| f.size).sum()
    }

    pub fn total_lines(&self) -> usize {
        self.files.values().map(|f| f.lines).sum()
    }

    pub fn symbol_count(&self) -> usize {
        self.files.values().map(|f| f.symbols.len()).sum()
    }

    /// Files whose path contains `query` (case-insensitive), sorted by path.
    pub fn search_paths(&self, query: &str) -> Vec<&FileRecord> {
        let needle = query.to_ascii_lowercase();
        let mut out: Vec<&FileRecord> = self
            .files
            .values()
            .filter(|f| f.path.to_ascii_lowercase().contains(&needle))
            .collect();
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }

    /// All symbols, optionally filtered by a case-insensitive name substring,
    /// each paired with the path it was found in. Sorted by name then path.
    pub fn search_symbols(&self, query: Option<&str>) -> Vec<(&str, &Symbol)> {
        let needle = query.map(|q| q.to_ascii_lowercase());
        let mut out: Vec<(&str, &Symbol)> = Vec::new();
        for file in self.files.values() {
            for sym in &file.symbols {
                let keep = match &needle {
                    Some(n) => sym.name.to_ascii_lowercase().contains(n),
                    None => true,
                };
                if keep {
                    out.push((file.path.as_str(), sym));
                }
            }
        }
        out.sort_by(|a, b| a.1.name.cmp(&b.1.name).then(a.0.cmp(b.0)));
        out
    }

    /// The `n` largest files by byte size, descending.
    pub fn largest_files(&self, n: usize) -> Vec<&FileRecord> {
        let mut all: Vec<&FileRecord> = self.files.values().collect();
        all.sort_by(|a, b| b.size.cmp(&a.size).then(a.path.cmp(&b.path)));
        all.truncate(n);
        all
    }

    /// Per-language breakdown: (file count, total lines), for files with a
    /// recognized language, keyed by language.
    pub fn language_breakdown(&self) -> BTreeMap<FileLang, (usize, usize)> {
        let mut out: BTreeMap<FileLang, (usize, usize)> = BTreeMap::new();
        for file in self.files.values() {
            if let Some(lang) = file.lang {
                let entry = out.entry(lang).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += file.lines;
            }
        }
        out
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Walk `root`, skipping `ignored` directories, returning each file's
/// `(relative-path, size, mtime-secs)` cheaply from directory metadata.
fn walk_files(root: &Path, ignored: &[String]) -> Vec<(String, u64, u64)> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_type().is_dir() {
                let name = entry.file_name().to_string_lossy();
                !ignored.iter().any(|d| d == name.as_ref())
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let rel = entry.path().strip_prefix(root).ok()?;
            let rel = normalize_rel(rel);
            let meta = entry.metadata().ok()?;
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Some((rel, meta.len(), mtime))
        })
        .collect()
}

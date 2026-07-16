//! Cleanup planning: find the regenerable cache/artifact directories present in
//! a project and measure how much space each occupies. Planning is read-only;
//! actually deleting is the caller's job, after confirmation.

use std::path::{Path, PathBuf};

use crate::fs_util::dir_size;

/// A regenerable directory Blink can offer to remove, with its measured size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanTarget {
    /// Directory name (e.g. `target`, `node_modules`).
    pub name: String,
    /// Absolute path to the directory.
    pub path: PathBuf,
    /// Measured size in bytes.
    pub bytes: u64,
    /// Whether this is a heavy build/dependency artifact (`--all` territory)
    /// rather than a lightweight cache. Dependencies like `node_modules` need a
    /// reinstall to restore, so they're only cleaned when explicitly requested.
    pub heavy: bool,
}

/// Candidate directories: (name, is_heavy). "Heavy" ones cost a reinstall/
/// recompile to regenerate, so `blink clean` leaves them unless `--all`.
const CANDIDATES: &[(&str, bool)] = &[
    ("target", true),
    ("node_modules", true),
    (".venv", true),
    ("venv", true),
    ("dist", false),
    ("build", false),
    (".next", false),
    (".turbo", false),
    (".cache", false),
    ("coverage", false),
    ("__pycache__", false),
    (".blink", false),
];

/// Plan a cleanup for `root`: every candidate directory that actually exists,
/// with its size measured. Order matches [`CANDIDATES`]. Nested `__pycache__`
/// directories deeper in the tree are found too.
pub fn plan(root: &Path) -> Vec<CleanTarget> {
    let mut out = Vec::new();

    for (name, heavy) in CANDIDATES {
        let path = root.join(name);
        if path.is_dir() {
            out.push(CleanTarget {
                name: (*name).to_string(),
                path: path.clone(),
                bytes: dir_size(&path),
                heavy: *heavy,
            });
        }
    }

    // Python leaves `__pycache__` scattered throughout the source tree, not
    // just at the root; collect the nested ones too (bounded depth to stay fast
    // and avoid descending into already-listed heavy dirs).
    for entry in walkdir::WalkDir::new(root)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir()
            && entry.file_name() == "__pycache__"
            && entry.path() != root.join("__pycache__")
        {
            // Skip anything inside a directory we've already listed wholesale.
            if out.iter().any(|t| entry.path().starts_with(&t.path)) {
                continue;
            }
            out.push(CleanTarget {
                name: "__pycache__".to_string(),
                path: entry.path().to_path_buf(),
                bytes: dir_size(entry.path()),
                heavy: false,
            });
        }
    }

    out
}

/// Total bytes across `targets`.
pub fn total_bytes(targets: &[CleanTarget]) -> u64 {
    targets.iter().map(|t| t.bytes).sum()
}

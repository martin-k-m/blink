//! Filesystem/storage breakdown: where a repository's bytes actually live, so a
//! developer can see that (say) `target/` dwarfs their source tree.

use std::path::Path;

use blink_core::effective_ignored_dirs;

use crate::fs_util::dir_size;

/// Storage used by one top-level entry in the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirUsage {
    pub name: String,
    pub bytes: u64,
    /// Whether this directory is on Blink's ignore list (build output,
    /// dependencies, caches) — i.e. regenerable weight, not source.
    pub ignored: bool,
    pub is_dir: bool,
}

/// A whole-project storage report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilesystemReport {
    /// Total bytes across everything under the root.
    pub total_bytes: u64,
    /// Bytes living in ignored (regenerable) directories.
    pub ignored_bytes: u64,
    /// Top-level entries, largest first.
    pub entries: Vec<DirUsage>,
}

impl FilesystemReport {
    /// Bytes that are actual project content (total minus regenerable weight).
    pub fn source_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.ignored_bytes)
    }
}

/// Measure storage under `root`, one line per top-level entry.
pub fn analyze(root: &Path) -> FilesystemReport {
    let ignored = effective_ignored_dirs(root);
    let mut entries = Vec::new();
    let mut total = 0u64;
    let mut ignored_total = 0u64;

    let read = match std::fs::read_dir(root) {
        Ok(r) => r,
        Err(_) => return FilesystemReport::default(),
    };

    for entry in read.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();
        let bytes = if is_dir {
            dir_size(&path)
        } else {
            std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        };
        let is_ignored = is_dir && ignored.iter().any(|d| d == &name);

        total += bytes;
        if is_ignored {
            ignored_total += bytes;
        }
        entries.push(DirUsage {
            name,
            bytes,
            ignored: is_ignored,
            is_dir,
        });
    }

    entries.sort_by(|a, b| b.bytes.cmp(&a.bytes).then(a.name.cmp(&b.name)));

    FilesystemReport {
        total_bytes: total,
        ignored_bytes: ignored_total,
        entries,
    }
}

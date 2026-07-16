use std::path::Path;

use walkdir::WalkDir;

/// Total size, in bytes, of every file under `path` (recursively). Unlike a
/// scan, this deliberately does *not* skip ignored directories — callers ask
/// for the size of things like `target/` or `node_modules/` precisely because
/// they're the big, ignorable artifacts.
pub fn dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Whether `name` resolves to an executable on the current `PATH`. Pure
/// filesystem lookup — nothing is spawned — so it's safe and fast to call for
/// many tools. On Windows the usual executable extensions are tried.
pub fn on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    let exts: &[&str] = if cfg!(windows) {
        &["", ".exe", ".cmd", ".bat"]
    } else {
        &[""]
    };
    for dir in std::env::split_paths(&path) {
        for ext in exts {
            let candidate = dir.join(format!("{name}{ext}"));
            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

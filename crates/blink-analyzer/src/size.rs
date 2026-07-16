use std::path::{Path, PathBuf};

use blink_core::{Language, Project};
use blink_parser::LockedPackage;
use walkdir::WalkDir;

/// A dependency and its measured installed size.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LargeDependency {
    pub name: String,
    pub bytes: u64,
}

/// Dependencies at or above this installed size are flagged as "large".
pub const LARGE_DEPENDENCY_THRESHOLD_BYTES: u64 = 5 * 1024 * 1024; // 5 MiB

/// Measure the installed size of each direct dependency and return the ones
/// over [`LARGE_DEPENDENCY_THRESHOLD_BYTES`], largest first.
///
/// Returns an empty list when dependencies haven't been installed/built
/// locally yet — there is nothing on disk to measure.
pub fn find_large_dependencies(
    project: &Project,
    root: &Path,
    locked: Option<&[LockedPackage]>,
) -> Vec<LargeDependency> {
    let mut large = sized_dependencies(project, root, locked);
    large.retain(|d| d.bytes >= LARGE_DEPENDENCY_THRESHOLD_BYTES);
    large
}

/// Return the `limit` largest direct dependencies by installed size,
/// regardless of whether they cross the "large" threshold. Used for the
/// "Largest Packages" section of a dependency report.
pub fn largest_dependencies(
    project: &Project,
    root: &Path,
    locked: Option<&[LockedPackage]>,
    limit: usize,
) -> Vec<LargeDependency> {
    let mut sized = sized_dependencies(project, root, locked);
    sized.truncate(limit);
    sized
}

/// Measure every direct dependency that's actually present on disk,
/// largest first. For JS/TS, that's `node_modules/<name>` (present once
/// `npm install` has run). For Rust, that's the crate's extracted source
/// under `$CARGO_HOME/registry/src/*/<name>-<resolved-version>` (present
/// once `cargo build` has downloaded it) — which requires the *resolved*
/// version from a lockfile, since the manifest's version requirement
/// (e.g. `"1"`) rarely matches the cache directory name exactly.
fn sized_dependencies(
    project: &Project,
    root: &Path,
    locked: Option<&[LockedPackage]>,
) -> Vec<LargeDependency> {
    let mut sized: Vec<LargeDependency> = match project.language {
        Language::TypeScript | Language::JavaScript => {
            let node_modules = root.join("node_modules");
            if !node_modules.is_dir() {
                return Vec::new();
            }
            project
                .dependencies
                .iter()
                .filter_map(|dep| {
                    let dep_dir = node_modules.join(&dep.name);
                    dep_dir.is_dir().then(|| LargeDependency {
                        name: dep.name.clone(),
                        bytes: dir_size(&dep_dir),
                    })
                })
                .collect()
        }
        Language::Rust => {
            let Some(locked) = locked else {
                return Vec::new();
            };
            project
                .dependencies
                .iter()
                .filter_map(|dep| {
                    let resolved_version = locked.iter().find(|p| p.name == dep.name)?;
                    let dir = find_cargo_registry_src_dir(&dep.name, &resolved_version.version)?;
                    Some(LargeDependency {
                        name: dep.name.clone(),
                        bytes: dir_size(&dir),
                    })
                })
                .collect()
        }
        Language::Python | Language::Unknown => Vec::new(),
    };

    sized.sort_by_key(|d| std::cmp::Reverse(d.bytes));
    sized
}

/// Locate `<name>-<version>`'s extracted source under any registry index
/// directory in the local Cargo cache.
fn find_cargo_registry_src_dir(name: &str, version: &str) -> Option<PathBuf> {
    let src_root = cargo_home_dir()?.join("registry").join("src");
    let index_dirs = std::fs::read_dir(&src_root).ok()?;
    for index_dir in index_dirs.flatten() {
        let candidate = index_dir.path().join(format!("{name}-{version}"));
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

fn cargo_home_dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CARGO_HOME") {
        return Some(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".cargo"))
}

/// The size of a project's most recent build output, if one exists.
pub fn build_output_size(project: &Project, root: &Path) -> Option<u64> {
    match project.language {
        Language::Rust => {
            let release_dir = root.join("target").join("release");
            release_dir.is_dir().then(|| dir_size(&release_dir))
        }
        Language::TypeScript | Language::JavaScript => {
            for candidate in ["dist", "build", ".next"] {
                let dir = root.join(candidate);
                if dir.is_dir() {
                    return Some(dir_size(&dir));
                }
            }
            None
        }
        Language::Python | Language::Unknown => None,
    }
}

fn dir_size(dir: &Path) -> u64 {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Format a byte count as a short human-readable string, e.g. `2.4MB`.
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1}GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes / KB)
    } else {
        format!("{bytes}B")
    }
}

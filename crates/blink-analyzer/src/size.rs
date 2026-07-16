use std::path::Path;

use blink_core::{Language, Project};
use walkdir::WalkDir;

/// A dependency whose on-disk footprint exceeds [`LARGE_DEPENDENCY_THRESHOLD_BYTES`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct LargeDependency {
    pub name: String,
    pub bytes: u64,
}

/// Dependencies at or above this installed size are flagged as "large".
pub const LARGE_DEPENDENCY_THRESHOLD_BYTES: u64 = 5 * 1024 * 1024; // 5 MiB

/// Measure the installed size of each direct dependency under `node_modules`
/// and return the ones over the large-dependency threshold, largest first.
///
/// Returns an empty list when dependencies haven't been installed yet
/// (there is nothing on disk to measure).
pub fn find_large_dependencies(project: &Project, root: &Path) -> Vec<LargeDependency> {
    if !matches!(
        project.language,
        Language::TypeScript | Language::JavaScript
    ) {
        return Vec::new();
    }

    let node_modules = root.join("node_modules");
    if !node_modules.is_dir() {
        return Vec::new();
    }

    let mut large: Vec<LargeDependency> = project
        .dependencies
        .iter()
        .filter_map(|dep| {
            let dep_dir = node_modules.join(&dep.name);
            if !dep_dir.is_dir() {
                return None;
            }
            let bytes = dir_size(&dep_dir);
            (bytes >= LARGE_DEPENDENCY_THRESHOLD_BYTES).then_some(LargeDependency {
                name: dep.name.clone(),
                bytes,
            })
        })
        .collect();

    large.sort_by_key(|d| std::cmp::Reverse(d.bytes));
    large
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

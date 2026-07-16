use std::path::Path;

use blink_core::{Framework, Language, Project};
use walkdir::WalkDir;

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

/// Scan a project's source files and return the names of declared
/// *runtime* (non-dev) dependencies that don't appear to be imported
/// anywhere. Dev dependencies are skipped: tools like `typescript` or
/// `eslint` are frequently invoked from config files or the CLI rather
/// than imported, so scanning for them produces false positives.
pub fn find_unused(project: &Project, root: &Path) -> Vec<String> {
    let runtime_deps: Vec<&str> = project
        .dependencies
        .iter()
        .filter(|d| !d.dev)
        .map(|d| d.name.as_str())
        .collect();

    if runtime_deps.is_empty() {
        return Vec::new();
    }

    let source = match project.language {
        Language::Rust => read_matching_sources(root, &["rs"]),
        Language::TypeScript | Language::JavaScript => read_matching_sources(
            root,
            &["ts", "tsx", "js", "jsx", "mjs", "cjs", "vue", "svelte"],
        ),
        Language::Python => read_matching_sources(root, &["py"]),
        Language::Unknown => String::new(),
    };

    runtime_deps
        .into_iter()
        .filter(|name| !is_referenced(project, name, &source))
        .map(str::to_string)
        .collect()
}

fn is_referenced(project: &Project, name: &str, source: &str) -> bool {
    match project.language {
        Language::Rust => {
            let ident = name.replace('-', "_");
            source.contains(&format!("use {ident}"))
                || source.contains(&format!("use {ident}::"))
                || source.contains(&format!("{ident}::"))
                || source.contains(&format!("extern crate {ident}"))
        }
        _ => source.contains(name),
    }
}

/// Frameworks whose dependency is a build-time/runtime marker rather than
/// something imported directly (e.g. `next` is invoked via the `next` CLI).
pub fn is_framework_marker(project: &Project, name: &str) -> bool {
    matches!(
        (project.framework, name),
        (Framework::NextJs, "next") | (Framework::Cargo, _)
    )
}

fn read_matching_sources(root: &Path, extensions: &[&str]) -> String {
    let mut combined = String::new();

    for entry in WalkDir::new(root)
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
    {
        let is_match = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| extensions.contains(&ext))
            .unwrap_or(false);

        if is_match {
            if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                combined.push_str(&contents);
                combined.push('\n');
            }
        }
    }

    combined
}

use std::path::Path;

use blink_parser::RawDependency;
use walkdir::WalkDir;

use crate::config::BlinkConfig;
use crate::error::{BlinkError, Result};
use crate::project::{Dependency, Framework, Language, PackageManager, Project};

/// Directory names that are never counted or descended into during a scan,
/// regardless of `blink.toml`. Projects can add to this list via
/// `[project].ignore`; see [`effective_ignored_dirs`].
pub const DEFAULT_IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".turbo",
    ".cache",
    ".blink",
    "coverage",
    "__pycache__",
    ".venv",
    "venv",
];

/// The directory names to skip while walking `root`: [`DEFAULT_IGNORED_DIRS`]
/// plus whatever `root`/`blink.toml`'s `[project].ignore` adds. Missing or
/// unparsable config is treated as "no extra ignores" rather than an error.
pub fn effective_ignored_dirs(root: &Path) -> Vec<String> {
    let mut dirs: Vec<String> = DEFAULT_IGNORED_DIRS.iter().map(|s| s.to_string()).collect();
    if let Ok(config) = BlinkConfig::load(root) {
        for entry in config.extra_ignores() {
            if !dirs.contains(&entry) {
                dirs.push(entry);
            }
        }
    }
    dirs
}

/// Detects the language, framework, package manager, and dependency set of a
/// project by inspecting its manifest files and directory layout.
#[derive(Debug, Default)]
pub struct ProjectDetector;

impl ProjectDetector {
    pub fn new() -> Self {
        Self
    }

    /// Scan `dir` and produce a [`Project`] describing what was found there.
    pub fn detect(&self, dir: &Path) -> Result<Project> {
        if !dir.exists() {
            return Err(BlinkError::PathNotFound(dir.to_path_buf()));
        }
        if !dir.is_dir() {
            return Err(BlinkError::NotADirectory(dir.to_path_buf()));
        }

        let ignored_dirs = effective_ignored_dirs(dir);
        let file_count = count_files(dir, &ignored_dirs);

        if dir.join("Cargo.toml").is_file() {
            return self.detect_rust(dir, file_count);
        }
        if dir.join("package.json").is_file() {
            return self.detect_node(dir, file_count);
        }
        if dir.join("requirements.txt").is_file() || dir.join("pyproject.toml").is_file() {
            return self.detect_python(dir, file_count);
        }

        Err(BlinkError::UnknownProject(dir.to_path_buf()))
    }

    fn detect_rust(&self, dir: &Path, file_count: usize) -> Result<Project> {
        let manifest_path = dir.join("Cargo.toml");
        let raw = std::fs::read_to_string(&manifest_path).map_err(|source| BlinkError::Io {
            path: manifest_path.clone(),
            source,
        })?;
        let manifest = blink_parser::parse_cargo_manifest(&raw);

        let name = manifest.name.unwrap_or_else(|| fallback_name(dir));
        let dependencies = manifest
            .dependencies
            .into_iter()
            .map(to_dependency)
            .collect();

        Ok(Project {
            name,
            root: dir.to_path_buf(),
            language: Language::Rust,
            framework: Framework::Cargo,
            package_manager: PackageManager::Cargo,
            dependencies,
            file_count,
            config_file: "Cargo.toml".to_string(),
            is_workspace: manifest.is_workspace,
            has_virtualenv: false,
        })
    }

    fn detect_node(&self, dir: &Path, file_count: usize) -> Result<Project> {
        let manifest_path = dir.join("package.json");
        let raw = std::fs::read_to_string(&manifest_path).map_err(|source| BlinkError::Io {
            path: manifest_path.clone(),
            source,
        })?;
        let manifest =
            blink_parser::parse_package_json(&raw).map_err(|source| BlinkError::ManifestParse {
                path: manifest_path.clone(),
                source,
            })?;

        // Vite is a build tool, not a UI framework, so it only gets reported
        // when nothing more specific was detected (e.g. a vanilla Vite
        // project) rather than overriding React/Vue/Svelte/Next, which are
        // commonly used *with* Vite.
        let framework = if manifest.has_dependency("next") {
            Framework::NextJs
        } else if manifest.has_dependency("react") {
            Framework::React
        } else if manifest.has_dependency("vue") {
            Framework::Vue
        } else if manifest.has_dependency("svelte") {
            Framework::Svelte
        } else if manifest.has_dependency("vite") {
            Framework::Vite
        } else {
            Framework::None
        };

        let language =
            if dir.join("tsconfig.json").is_file() || manifest.has_dependency("typescript") {
                Language::TypeScript
            } else {
                Language::JavaScript
            };

        let package_manager = if dir.join("pnpm-lock.yaml").is_file() {
            PackageManager::Pnpm
        } else if dir.join("yarn.lock").is_file() {
            PackageManager::Yarn
        } else {
            // package-lock.json or no lockfile at all: npm is the default.
            PackageManager::Npm
        };

        let name = manifest.name.clone().unwrap_or_else(|| fallback_name(dir));
        let dependencies = manifest
            .dependencies
            .into_iter()
            .map(to_dependency)
            .collect();

        Ok(Project {
            name,
            root: dir.to_path_buf(),
            language,
            framework,
            package_manager,
            dependencies,
            file_count,
            config_file: "package.json".to_string(),
            is_workspace: false,
            has_virtualenv: false,
        })
    }

    fn detect_python(&self, dir: &Path, file_count: usize) -> Result<Project> {
        let mut dependencies = Vec::new();

        let requirements_path = dir.join("requirements.txt");
        let config_file = if requirements_path.is_file() {
            let raw =
                std::fs::read_to_string(&requirements_path).map_err(|source| BlinkError::Io {
                    path: requirements_path.clone(),
                    source,
                })?;
            dependencies.extend(
                blink_parser::parse_requirements_txt(&raw)
                    .into_iter()
                    .map(to_dependency),
            );
            "requirements.txt".to_string()
        } else {
            "pyproject.toml".to_string()
        };

        // v0.1 doesn't yet distinguish pip/poetry/pipenv, so this is always Pip.
        let package_manager = PackageManager::Pip;
        let has_virtualenv = dir.join(".venv").is_dir() || dir.join("venv").is_dir();

        Ok(Project {
            name: fallback_name(dir),
            root: dir.to_path_buf(),
            language: Language::Python,
            framework: Framework::None,
            package_manager,
            dependencies,
            file_count,
            config_file,
            is_workspace: false,
            has_virtualenv,
        })
    }
}

fn to_dependency(raw: RawDependency) -> Dependency {
    Dependency {
        name: raw.name,
        version: raw.version,
        dev: raw.dev,
    }
}

fn fallback_name(dir: &Path) -> String {
    let absolute = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    absolute
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed-project".to_string())
}

/// Count all files under `dir`, skipping directories named in `ignored_dirs`.
fn count_files(dir: &Path, ignored_dirs: &[String]) -> usize {
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_type().is_dir() {
                let name = entry.file_name().to_string_lossy();
                !ignored_dirs.iter().any(|d| d == name.as_ref())
            } else {
                true
            }
        })
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .count()
}

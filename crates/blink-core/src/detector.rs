use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use walkdir::WalkDir;

use crate::error::{BlinkError, Result};
use crate::project::{Dependency, Framework, Language, PackageManager, Project};

/// Directory names that are never counted or descended into during a scan.
const IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".turbo",
    ".cache",
    ".blink",
    "__pycache__",
    ".venv",
    "venv",
];

/// Detects the language, framework, package manager, and dependency set of a
/// project by inspecting its manifest files and directory layout.
#[derive(Debug, Default)]
pub struct ProjectDetector;

#[derive(Debug, Deserialize, Default)]
struct PackageJson {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(default)]
    #[serde(rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
struct CargoManifest {
    #[serde(default)]
    package: Option<CargoPackage>,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
    #[serde(default)]
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Deserialize, Default)]
struct CargoPackage {
    name: Option<String>,
}

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

        let file_count = count_files(dir);

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
        let manifest: CargoManifest = toml::from_str(&raw).unwrap_or_default();

        let name = manifest
            .package
            .and_then(|p| p.name)
            .unwrap_or_else(|| fallback_name(dir));

        let mut dependencies: Vec<Dependency> = manifest
            .dependencies
            .into_iter()
            .map(|(name, value)| Dependency {
                name,
                version: cargo_dep_version(&value),
                dev: false,
            })
            .collect();
        dependencies.extend(manifest.dev_dependencies.into_iter().map(|(name, value)| {
            Dependency {
                name,
                version: cargo_dep_version(&value),
                dev: true,
            }
        }));
        dependencies.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(Project {
            name,
            root: dir.to_path_buf(),
            language: Language::Rust,
            framework: Framework::Cargo,
            package_manager: PackageManager::Cargo,
            dependencies,
            file_count,
        })
    }

    fn detect_node(&self, dir: &Path, file_count: usize) -> Result<Project> {
        let manifest_path = dir.join("package.json");
        let raw = std::fs::read_to_string(&manifest_path).map_err(|source| BlinkError::Io {
            path: manifest_path.clone(),
            source,
        })?;
        let manifest: PackageJson =
            serde_json::from_str(&raw).map_err(|source| BlinkError::ManifestParse {
                path: manifest_path.clone(),
                source,
            })?;

        let mut dependencies: Vec<Dependency> = manifest
            .dependencies
            .iter()
            .map(|(name, version)| Dependency {
                name: name.clone(),
                version: version.clone(),
                dev: false,
            })
            .collect();
        dependencies.extend(
            manifest
                .dev_dependencies
                .iter()
                .map(|(name, version)| Dependency {
                    name: name.clone(),
                    version: version.clone(),
                    dev: true,
                }),
        );
        dependencies.sort_by(|a, b| a.name.cmp(&b.name));

        let has_dep = |name: &str| {
            manifest.dependencies.contains_key(name) || manifest.dev_dependencies.contains_key(name)
        };

        let framework = if has_dep("next") {
            Framework::NextJs
        } else if has_dep("react") {
            Framework::React
        } else if has_dep("vue") {
            Framework::Vue
        } else if has_dep("svelte") {
            Framework::Svelte
        } else {
            Framework::None
        };

        let language = if dir.join("tsconfig.json").is_file() || has_dep("typescript") {
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

        let name = manifest.name.unwrap_or_else(|| fallback_name(dir));

        Ok(Project {
            name,
            root: dir.to_path_buf(),
            language,
            framework,
            package_manager,
            dependencies,
            file_count,
        })
    }

    fn detect_python(&self, dir: &Path, file_count: usize) -> Result<Project> {
        let mut dependencies = Vec::new();

        let requirements_path = dir.join("requirements.txt");
        if requirements_path.is_file() {
            let raw =
                std::fs::read_to_string(&requirements_path).map_err(|source| BlinkError::Io {
                    path: requirements_path.clone(),
                    source,
                })?;
            dependencies.extend(parse_requirements(&raw));
        }

        // v0.1 doesn't yet distinguish pip/poetry/pipenv, so this is always Pip.
        let package_manager = PackageManager::Pip;

        Ok(Project {
            name: fallback_name(dir),
            root: dir.to_path_buf(),
            language: Language::Python,
            framework: Framework::None,
            package_manager,
            dependencies,
            file_count,
        })
    }
}

fn cargo_dep_version(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string(),
        _ => "*".to_string(),
    }
}

fn parse_requirements(raw: &str) -> Vec<Dependency> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (name, version) = line
                .split_once("==")
                .or_else(|| line.split_once(">="))
                .or_else(|| line.split_once("~="))
                .unwrap_or((line, "*"));
            Dependency {
                name: name.trim().to_string(),
                version: version.trim().to_string(),
                dev: false,
            }
        })
        .collect()
}

fn fallback_name(dir: &Path) -> String {
    let absolute = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    absolute
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed-project".to_string())
}

/// Count all files under `dir`, skipping directories in [`IGNORED_DIRS`].
fn count_files(dir: &Path) -> usize {
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_type().is_dir() {
                let name = entry.file_name().to_string_lossy();
                !IGNORED_DIRS.contains(&name.as_ref())
            } else {
                true
            }
        })
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .count()
}

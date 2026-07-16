use std::fmt;
use std::path::PathBuf;

/// A primary programming language detected in a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Unknown,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Unknown => "Unknown",
        };
        write!(f, "{s}")
    }
}

/// A web/application framework detected via manifest dependencies or file conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Framework {
    React,
    NextJs,
    Vue,
    Svelte,
    Cargo,
    None,
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Framework::React => "React",
            Framework::NextJs => "Next.js",
            Framework::Vue => "Vue",
            Framework::Svelte => "Svelte",
            Framework::Cargo => "Cargo",
            Framework::None => "None",
        };
        write!(f, "{s}")
    }
}

/// A package manager detected via lockfiles or manifest tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Cargo,
    Pip,
    Unknown,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PackageManager::Npm => "npm",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Yarn => "yarn",
            PackageManager::Cargo => "cargo",
            PackageManager::Pip => "pip",
            PackageManager::Unknown => "unknown",
        };
        write!(f, "{s}")
    }
}

/// A single declared dependency, as found in a project manifest.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub dev: bool,
}

/// The result of scanning a directory: everything Blink was able to determine
/// about the project living there.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub name: String,
    pub root: PathBuf,
    pub language: Language,
    pub framework: Framework,
    pub package_manager: PackageManager,
    pub dependencies: Vec<Dependency>,
    pub file_count: usize,
}

impl Project {
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }
}

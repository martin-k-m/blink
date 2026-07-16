//! Core types shared across Blink: project detection, configuration, and errors.

mod config;
mod detector;
mod error;
mod project;

#[cfg(test)]
mod tests;

pub use config::{BlinkConfig, OptimizationConfig, ProjectConfig, ServerConfig, CONFIG_FILE_NAME};
pub use detector::{effective_ignored_dirs, ProjectDetector, DEFAULT_IGNORED_DIRS};
pub use error::{BlinkError, Result};
pub use project::{Dependency, Framework, Language, PackageManager, Project};

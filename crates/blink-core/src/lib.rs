//! Core types shared across Blink: project detection, configuration, and errors.

mod config;
mod detector;
mod error;
mod project;

#[cfg(test)]
mod tests;

pub use config::{
    BlinkConfig, ContextConfig, IndexConfig, OptimizationConfig, ProfileConfig, ProjectConfig,
    ReportConfig, ScanConfig, ServerConfig, CONFIG_FILE_NAME, CONFIG_FILE_NAMES,
    SIGNATURE_CONFIG_FILE_NAME,
};
pub use detector::{effective_ignored_dirs, ProjectDetector, DEFAULT_IGNORED_DIRS};
pub use error::{BlinkError, Result};
pub use project::{Dependency, Framework, Language, PackageManager, Project};

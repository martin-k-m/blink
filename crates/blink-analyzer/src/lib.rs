//! Dependency graph construction and project health analysis for Blink.

mod dependencies;
mod duplicates;
mod graph;
mod health;
mod lockfile;
mod recommendations;
mod registry;
mod report;
mod security;
mod size;
mod usage;
mod version;

#[cfg(test)]
mod tests;

pub use dependencies::DependencyCounts;
pub use duplicates::DuplicatePackage;
pub use graph::{DependencyGraph, DependencyNode};
pub use health::{compute_health, HealthReport};
pub use recommendations::{Recommendation, RecommendationCategory, RecommendationEngine, Status};
pub use registry::OutdatedPackage;
pub use report::{AnalysisReport, Analyzer};
pub use security::{find_vulnerabilities, VulnerablePackage};
pub use size::{format_bytes, LargeDependency, LARGE_DEPENDENCY_THRESHOLD_BYTES};

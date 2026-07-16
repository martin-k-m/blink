//! Dependency graph construction and project health analysis for Blink.

mod duplicates;
mod graph;
mod registry;
mod report;
mod size;
mod usage;

#[cfg(test)]
mod tests;

pub use duplicates::DuplicatePackage;
pub use graph::{DependencyGraph, DependencyNode};
pub use registry::OutdatedPackage;
pub use report::{AnalysisReport, Analyzer};
pub use size::{format_bytes, LargeDependency, LARGE_DEPENDENCY_THRESHOLD_BYTES};

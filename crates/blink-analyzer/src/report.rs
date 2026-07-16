use std::path::Path;
use std::time::Instant;

use blink_core::Project;

use crate::duplicates::{self, DuplicatePackage};
use crate::graph::DependencyGraph;
use crate::registry::{self, OutdatedPackage};
use crate::size::{self, LargeDependency};
use crate::usage;

/// The full result of analyzing a project's dependency health.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalysisReport {
    pub dependency_graph: DependencyGraph,
    pub unused: Vec<String>,
    pub duplicates: Vec<DuplicatePackage>,
    pub large_dependencies: Vec<LargeDependency>,
    pub outdated: Vec<OutdatedPackage>,
    pub outdated_checked: bool,
    pub build_output_bytes: Option<u64>,
    pub elapsed_ms: u128,
}

impl AnalysisReport {
    /// Number of direct dependencies with no detected issues.
    pub fn healthy_count(&self) -> usize {
        let flagged: usize = self.unused.len() + self.duplicates.len() + self.outdated.len();
        self.dependency_graph.direct_count().saturating_sub(flagged)
    }

    /// Actionable recommendations derived strictly from what was found;
    /// nothing here is generic filler.
    pub fn recommendations(&self) -> Vec<String> {
        let mut recs = Vec::new();

        for name in &self.unused {
            recs.push(format!("Remove unused package {name}"));
        }
        for dup in &self.duplicates {
            recs.push(format!(
                "Deduplicate {} ({} versions resolved: {})",
                dup.name,
                dup.versions.len(),
                dup.versions.join(", ")
            ));
        }
        for large in &self.large_dependencies {
            recs.push(format!(
                "Review large dependency {} ({})",
                large.name,
                size::format_bytes(large.bytes)
            ));
        }
        for out in &self.outdated {
            recs.push(format!(
                "Upgrade {} from {} to {}",
                out.name, out.current, out.latest
            ));
        }

        recs
    }
}

/// Analyzes a detected [`Project`] for dependency health issues.
#[derive(Debug, Default)]
pub struct Analyzer {
    /// When true, queries crates.io / the npm registry for outdated
    /// package checks. Requires network access.
    pub online: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    pub fn analyze(&self, project: &Project, root: &Path) -> AnalysisReport {
        let start = Instant::now();

        let dependency_graph = DependencyGraph::from_project(project);
        let unused = usage::find_unused(project, root)
            .into_iter()
            .filter(|name| !usage::is_framework_marker(project, name))
            .collect();
        let duplicates = duplicates::find_duplicates(root);
        let large_dependencies = size::find_large_dependencies(project, root);
        let build_output_bytes = size::build_output_size(project, root);
        let outdated = if self.online {
            registry::find_outdated(project)
        } else {
            Vec::new()
        };

        AnalysisReport {
            dependency_graph,
            unused,
            duplicates,
            large_dependencies,
            outdated,
            outdated_checked: self.online,
            build_output_bytes,
            elapsed_ms: start.elapsed().as_millis(),
        }
    }
}

use std::path::Path;
use std::time::Instant;

use blink_core::Project;

use crate::dependencies::{self, DependencyCounts};
use crate::duplicates::{self, DuplicatePackage};
use crate::graph::DependencyGraph;
use crate::lockfile;
use crate::registry::{self, OutdatedPackage};
use crate::size::{self, LargeDependency};
use crate::usage;

/// The full result of analyzing a project's dependency health.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisReport {
    pub dependency_graph: DependencyGraph,
    pub dependency_counts: DependencyCounts,
    pub largest_dependencies: Vec<LargeDependency>,
    pub unused: Vec<String>,
    pub duplicates: Vec<DuplicatePackage>,
    pub large_dependencies: Vec<LargeDependency>,
    pub outdated: Vec<OutdatedPackage>,
    pub outdated_checked: bool,
    pub build_output_bytes: Option<u64>,
    pub elapsed_ms: u128,
}

/// How many entries `largest_dependencies` reports at most.
const LARGEST_DEPENDENCIES_LIMIT: usize = 5;

impl AnalysisReport {
    /// Number of direct dependencies with no detected issues.
    pub fn healthy_count(&self) -> usize {
        let flagged: usize = self.unused.len() + self.duplicates.len() + self.outdated.len();
        self.dependency_graph.direct_count().saturating_sub(flagged)
    }

    /// A 0-100 heuristic health score. This is a simple weighted deduction,
    /// not a rigorous metric: it exists to give an at-a-glance signal, not
    /// a precise measurement. See `docs/analysis.md` for the exact weights.
    pub fn health_score(&self) -> u8 {
        let mut score: i32 = 100;
        score -= (self.unused.len() as i32) * 5;
        score -= (self.duplicates.len() as i32) * 4;
        score -= (self.large_dependencies.len() as i32) * 3;
        if self.outdated_checked {
            score -= (self.outdated.len() as i32) * 2;
        }
        score.clamp(0, 100) as u8
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

        let locked = lockfile::load_locked_packages(root);

        let dependency_graph = DependencyGraph::from_project(project);
        let dependency_counts =
            dependencies::count_dependencies(dependency_graph.direct_count(), locked.as_deref());
        let unused = usage::find_unused(project, root)
            .into_iter()
            .filter(|name| !usage::is_framework_marker(project, name))
            .collect();
        let duplicates = locked
            .as_deref()
            .map(duplicates::find_duplicates)
            .unwrap_or_default();
        let large_dependencies = size::find_large_dependencies(project, root, locked.as_deref());
        let largest_dependencies = size::largest_dependencies(
            project,
            root,
            locked.as_deref(),
            LARGEST_DEPENDENCIES_LIMIT,
        );
        let build_output_bytes = size::build_output_size(project, root);
        let outdated = if self.online {
            registry::find_outdated(project)
        } else {
            Vec::new()
        };

        AnalysisReport {
            dependency_graph,
            dependency_counts,
            largest_dependencies,
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

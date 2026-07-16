//! The project context graph: a structured, serializable model of what a
//! project *is* — its identity, its areas, its files and the symbols they
//! declare, its declared dependencies, its runnable commands, and the
//! references that connect files to one another.
//!
//! Everything in this model is measured or resolved from real files. The only
//! interpretive step is how files are grouped into [`Area`]s (a presentation
//! choice, documented on [`area_of`](crate::build::area_of)); every count,
//! size, symbol, and reference is a concrete fact.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The complete context graph for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGraph {
    pub project: ProjectInfo,
    pub stats: Stats,
    pub config: ConfigInfo,
    /// Logical areas (directory groupings), ranked most-significant first.
    pub areas: Vec<Area>,
    pub dependencies: Vec<DependencyNode>,
    pub commands: Vec<CommandNode>,
    pub files: Vec<FileNode>,
    /// Resolved file→file references (an import in `from` pointing at `to`),
    /// both project-relative paths present in [`files`](ContextGraph::files).
    pub references: Vec<Reference>,
    /// The project root. Restored from the build location, never serialized, so
    /// an exported graph carries no absolute machine paths.
    #[serde(skip)]
    pub root: PathBuf,
}

/// Project identity, from `blink-core`'s detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub language: String,
    /// `None` when no application framework was detected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    pub package_manager: String,
    pub is_workspace: bool,
}

/// Whole-project totals, summed from the index.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Stats {
    /// Every indexed file (source and otherwise).
    pub files: usize,
    /// Files with a recognized source language (from which symbols come).
    pub source_files: usize,
    pub lines: usize,
    pub symbols: usize,
    pub size_bytes: u64,
}

/// What Blink knows about the project's own configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    /// Whether a `.bnk` or `blink.toml` was found.
    pub present: bool,
    /// The config filename found, if any (`.bnk` or `blink.toml`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Effective `[context].enabled` (defaults true).
    pub context_enabled: bool,
    /// Effective `[context].include` roots, empty meaning "the whole project".
    pub include: Vec<String>,
}

/// A logical area of the project — a directory grouping and its aggregate
/// measurements. See [`area_of`](crate::build::area_of) for the grouping rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub path: String,
    pub files: usize,
    pub lines: usize,
    pub symbols: usize,
    pub size_bytes: u64,
    /// Distinct source languages present, sorted.
    pub languages: Vec<String>,
}

/// A declared external dependency (direct, from the manifest).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub name: String,
    pub version: String,
    pub dev: bool,
}

/// A runnable command discovered for the project (task, script, recipe, ...).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandNode {
    pub name: String,
    pub command: String,
    pub source: String,
}

/// One top-level symbol declared in a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRef {
    pub name: String,
    pub kind: String,
    pub line: usize,
}

/// A single file in the graph, with the symbols it declares.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: String,
    pub area: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub lines: usize,
    pub size_bytes: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbols: Vec<SymbolRef>,
}

/// A resolved reference from one project file to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub from: String,
    pub to: String,
}

/// An aggregated dependency between two areas: how many file references cross
/// from `from` into `to`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaEdge {
    pub from: String,
    pub to: String,
    pub count: usize,
}

impl ContextGraph {
    /// Look up a single file node by its project-relative path.
    pub fn file(&self, path: &str) -> Option<&FileNode> {
        self.files.iter().find(|f| f.path == path)
    }

    /// Areas ranked by significance: most symbols first, then most lines, then
    /// path for a stable order.
    pub fn ranked_areas(&self) -> Vec<&Area> {
        let mut out: Vec<&Area> = self.areas.iter().collect();
        out.sort_by(|a, b| {
            b.symbols
                .cmp(&a.symbols)
                .then(b.lines.cmp(&a.lines))
                .then(a.path.cmp(&b.path))
        });
        out
    }

    /// Project files that `path` references (its outgoing edges), sorted.
    pub fn dependencies_of(&self, path: &str) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .references
            .iter()
            .filter(|r| r.from == path)
            .map(|r| r.to.as_str())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Project files that reference `path` (its incoming edges), sorted.
    pub fn dependents_of(&self, path: &str) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .references
            .iter()
            .filter(|r| r.to == path)
            .map(|r| r.from.as_str())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Aggregate the file→file references into area→area edges, dropping edges
    /// within a single area. Sorted by descending crossing count.
    pub fn area_edges(&self) -> Vec<AreaEdge> {
        let area_of: BTreeMap<&str, &str> = self
            .files
            .iter()
            .map(|f| (f.path.as_str(), f.area.as_str()))
            .collect();

        let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
        for r in &self.references {
            let (Some(from), Some(to)) = (area_of.get(r.from.as_str()), area_of.get(r.to.as_str()))
            else {
                continue;
            };
            if from == to {
                continue;
            }
            *counts
                .entry((from.to_string(), to.to_string()))
                .or_insert(0) += 1;
        }

        let mut out: Vec<AreaEdge> = counts
            .into_iter()
            .map(|((from, to), count)| AreaEdge { from, to, count })
            .collect();
        out.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then(a.from.cmp(&b.from))
                .then(a.to.cmp(&b.to))
        });
        out
    }
}

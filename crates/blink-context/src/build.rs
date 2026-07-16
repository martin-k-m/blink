//! Assembling a [`ContextGraph`] from the project's detected identity, its
//! index, and its configuration. The builder reads source files once (in
//! parallel) to resolve references; everything else is a re-shaping of data the
//! index and detector already computed.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use rayon::prelude::*;

use blink_core::{BlinkConfig, Framework, Project, ProjectDetector};
use blink_index::{FileLang, FileRecord, Index};
use blink_workflow::tasks;

use crate::imports;
use crate::model::{
    Area, CommandNode, ConfigInfo, ContextGraph, DependencyNode, FileNode, ProjectInfo, Reference,
    Stats, SymbolRef,
};

/// Directory names that act as containers rather than areas of their own: when
/// a file lives under one of these, its area is the container plus the next
/// path segment (e.g. `src/auth`, `crates/blink-core`) rather than just `src`.
const CONTAINER_DIRS: &[&str] = &[
    "src", "lib", "libs", "app", "apps", "crates", "packages", "pkg", "internal", "cmd", "source",
    "modules",
];

/// The logical area a project-relative `path` belongs to.
///
/// This is the one interpretive choice in the graph — a *grouping* for
/// presentation, not a measurement. The rule: a top-level file is `(root)`; a
/// file directly under a container directory (`src/main.rs`) groups by that
/// container; a file nested under a container (`src/auth/login.rs`,
/// `crates/blink-core/src/lib.rs`) groups by container plus its next segment;
/// anything else groups by its top-level directory.
pub fn area_of(path: &str) -> String {
    let segs: Vec<&str> = path.split('/').collect();
    match segs.len() {
        0 | 1 => "(root)".to_string(),
        2 => segs[0].to_string(),
        _ => {
            if CONTAINER_DIRS.contains(&segs[0]) {
                format!("{}/{}", segs[0], segs[1])
            } else {
                segs[0].to_string()
            }
        }
    }
}

impl ContextGraph {
    /// Build the full context graph for `root`, detecting the project and
    /// refreshing its index along the way.
    pub fn build(root: &Path) -> blink_core::Result<Self> {
        let project = ProjectDetector::new().detect(root)?;
        let (index, _) = Index::refresh(root).map_err(|source| blink_core::BlinkError::Io {
            path: root.join(".blink"),
            source: std::io::Error::other(source),
        })?;
        let config = BlinkConfig::load(root).ok();
        Ok(Self::from_parts(root, &project, &index, config.as_ref()))
    }

    /// Assemble a graph from already-computed parts (the CLI's warmed index and
    /// detected project). Reads source files to resolve references.
    pub fn from_parts(
        root: &Path,
        project: &Project,
        index: &Index,
        config: Option<&BlinkConfig>,
    ) -> Self {
        let ctx_cfg = config.map(|c| c.context.clone()).unwrap_or_default();
        let include = ctx_cfg.include.clone();
        let context_enabled = ctx_cfg.enabled;

        // The set of files this graph covers, honoring `[context].include`.
        let records: Vec<&FileRecord> = index
            .files
            .values()
            .filter(|r| included(&r.path, &include))
            .collect();
        let path_set: HashSet<String> = records.iter().map(|r| r.path.clone()).collect();

        // File nodes + running totals.
        let mut files: Vec<FileNode> = Vec::with_capacity(records.len());
        let mut stats = Stats::default();
        for rec in &records {
            stats.files += 1;
            stats.lines += rec.lines;
            stats.symbols += rec.symbols.len();
            stats.size_bytes += rec.size;
            if rec.lang.is_some() {
                stats.source_files += 1;
            }
            files.push(FileNode {
                path: rec.path.clone(),
                area: area_of(&rec.path),
                lang: rec.lang.map(|l| l.to_string()),
                lines: rec.lines,
                size_bytes: rec.size,
                symbols: rec
                    .symbols
                    .iter()
                    .map(|s| SymbolRef {
                        name: s.name.clone(),
                        kind: s.kind.to_string(),
                        line: s.line,
                    })
                    .collect(),
            });
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));

        let areas = aggregate_areas(&files);
        let references = scan_references(root, &records, &path_set);

        let dependencies: Vec<DependencyNode> = project
            .dependencies
            .iter()
            .map(|d| DependencyNode {
                name: d.name.clone(),
                version: d.version.clone(),
                dev: d.dev,
            })
            .collect();

        let commands: Vec<CommandNode> = tasks::discover(root)
            .into_iter()
            .map(|t| CommandNode {
                name: t.name,
                command: t.command,
                source: t.source.to_string(),
            })
            .collect();

        let (config_present, config_file) = match BlinkConfig::config_path(root) {
            Some(p) => (
                true,
                p.file_name().map(|n| n.to_string_lossy().into_owned()),
            ),
            None => (false, None),
        };

        let framework = match project.framework {
            Framework::None => None,
            other => Some(other.to_string()),
        };

        ContextGraph {
            project: ProjectInfo {
                name: project.name.clone(),
                language: project.language.to_string(),
                framework,
                package_manager: project.package_manager.to_string(),
                is_workspace: project.is_workspace,
            },
            stats,
            config: ConfigInfo {
                present: config_present,
                file: config_file,
                context_enabled,
                include,
            },
            areas,
            dependencies,
            commands,
            files,
            references,
            root: root.to_path_buf(),
        }
    }
}

/// Whether `path` is covered by an `include` list (empty means "everything").
/// A prefix matches on a path-segment boundary, so `src` includes `src/a.rs`
/// but not `srcfoo/a.rs`.
fn included(path: &str, include: &[String]) -> bool {
    if include.is_empty() {
        return true;
    }
    include.iter().any(|inc| {
        let inc = inc.trim_end_matches('/');
        path == inc || path.starts_with(&format!("{inc}/"))
    })
}

fn aggregate_areas(files: &[FileNode]) -> Vec<Area> {
    let mut map: BTreeMap<String, AreaAcc> = BTreeMap::new();
    for f in files {
        let acc = map.entry(f.area.clone()).or_default();
        acc.files += 1;
        acc.lines += f.lines;
        acc.symbols += f.symbols.len();
        acc.size_bytes += f.size_bytes;
        if let Some(lang) = &f.lang {
            acc.languages.insert(lang.clone());
        }
    }
    map.into_iter()
        .map(|(path, acc)| Area {
            path,
            files: acc.files,
            lines: acc.lines,
            symbols: acc.symbols,
            size_bytes: acc.size_bytes,
            languages: acc.languages.into_iter().collect(),
        })
        .collect()
}

#[derive(Default)]
struct AreaAcc {
    files: usize,
    lines: usize,
    symbols: usize,
    size_bytes: u64,
    languages: BTreeSet<String>,
}

/// Read every source file once (in parallel) and resolve its references to
/// other project files. For Rust, any in-workspace `<crate>::…` path (whether
/// in a `use` or an inline call) is resolved to that crate's library root via
/// [`rust_crate_map`], so cross-crate dependencies in a Cargo workspace show up
/// as real edges — not just the intra-crate `mod` tree.
fn scan_references(
    root: &Path,
    records: &[&FileRecord],
    paths: &HashSet<String>,
) -> Vec<Reference> {
    let crate_map = rust_crate_map(root, records, paths);

    let mut refs: Vec<Reference> = records
        .par_iter()
        .filter_map(|rec| {
            let lang = rec.lang?;
            let text = std::fs::read_to_string(root.join(&rec.path)).ok()?;
            let scanned = imports::scan(lang, &text, &rec.path, paths);
            let mut out: Vec<Reference> = scanned
                .internal
                .into_iter()
                .filter(|to| to != &rec.path) // ignore self-references
                .map(|to| Reference {
                    from: rec.path.clone(),
                    to,
                })
                .collect();
            if lang == FileLang::Rust && !crate_map.is_empty() {
                for ident in rust_path_idents(&text) {
                    if let Some(target) = crate_map.get(ident.as_str()) {
                        if target != &rec.path {
                            out.push(Reference {
                                from: rec.path.clone(),
                                to: target.clone(),
                            });
                        }
                    }
                }
            }
            Some(out)
        })
        .flatten()
        .collect();
    refs.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
    refs.dedup();
    refs
}

/// Every identifier that appears immediately before a `::` in `text` — the
/// leading segment of a path like `blink_core::Foo` or `monorepo_lib::add(…)`.
/// Used to spot references to sibling workspace crates whether they come via a
/// `use` or an inline path expression.
fn rust_path_idents(text: &str) -> HashSet<String> {
    let bytes = text.as_bytes();
    let mut out = HashSet::new();
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find("::") {
        let colon = search_from + rel;
        let mut start = colon;
        while start > 0 {
            let c = bytes[start - 1];
            if c.is_ascii_alphanumeric() || c == b'_' {
                start -= 1;
            } else {
                break;
            }
        }
        if start < colon {
            // A leading digit means it isn't an identifier (e.g. a numeric
            // literal); crate names never start with a digit.
            if !bytes[start].is_ascii_digit() {
                out.insert(text[start..colon].to_string());
            }
        }
        search_from = colon + 2;
    }
    out
}

/// Map each in-workspace crate's package name (with `-` normalized to `_`, as
/// `use` paths spell it) to its library root file — `<crate>/src/lib.rs`, or
/// `main.rs` for a binary-only crate. Only crates whose root is actually in the
/// index are included, so an edge always points at a real file.
fn rust_crate_map(
    root: &Path,
    records: &[&FileRecord],
    paths: &HashSet<String>,
) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for rec in records {
        if rec.path != "Cargo.toml" && !rec.path.ends_with("/Cargo.toml") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(&rec.path)) else {
            continue;
        };
        let Ok(value) = text.parse::<toml::Value>() else {
            continue;
        };
        let Some(name) = value
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
        else {
            continue;
        };
        let dir = match rec.path.strip_suffix("Cargo.toml") {
            Some(prefix) => prefix.trim_end_matches('/'),
            None => continue,
        };
        let lib = join(dir, "src/lib.rs");
        let main = join(dir, "src/main.rs");
        let rep = if paths.contains(&lib) {
            lib
        } else if paths.contains(&main) {
            main
        } else {
            continue;
        };
        map.insert(name.replace('-', "_"), rep);
    }
    map
}

fn join(dir: &str, rel: &str) -> String {
    if dir.is_empty() {
        rel.to_string()
    } else {
        format!("{dir}/{rel}")
    }
}

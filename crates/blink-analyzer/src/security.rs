use std::collections::{HashMap, HashSet, VecDeque};

use blink_core::{Language, Project};
use serde::{Deserialize, Serialize};

use crate::lockfile::{load_lockfile_from, ResolvedLockfile};
use crate::version::clean_version;

/// How many `(package, version)` pairs go into one OSV `querybatch` request.
/// OSV documents a 1000-query ceiling per batch; 500 keeps request bodies
/// small while still auditing a typical `Cargo.lock` in a single round trip.
const BATCH_SIZE: usize = 500;

/// OSV ecosystem name and the lockfiles that belong to it. A project is only
/// ever audited against the lockfile of its own ecosystem.
fn ecosystem_for(language: Language) -> Option<(&'static str, &'static [&'static str])> {
    match language {
        Language::Rust => Some(("crates.io", &["Cargo.lock"])),
        Language::TypeScript | Language::JavaScript => {
            Some(("npm", &["package-lock.json", "yarn.lock", "pnpm-lock.yaml"]))
        }
        // PyPI is a valid OSV ecosystem, but Blink doesn't read any Python
        // lockfile format, so a Python audit can only ever cover declared
        // requirements.
        Language::Python => Some(("PyPI", &[])),
        Language::Unknown => None,
    }
}

/// What the audit actually covered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuditScope {
    /// Every package a lockfile resolved — the full transitive graph.
    Lockfile {
        file: &'static str,
        /// Whether the format records package-to-package edges. When false,
        /// dependency paths can't be reconstructed and aren't shown.
        records_edges: bool,
    },
    /// Only the versions declared in the project manifest. Transitive
    /// dependencies are **not** covered; there was no lockfile to read.
    DeclaredOnly,
}

/// Whether the audit reached a conclusion at all. This is deliberately
/// separate from "found nothing": an unreachable advisory source must never
/// be reported as a clean bill of health.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AuditStatus {
    /// Every audited package was checked against OSV.dev.
    Completed,
    /// OSV.dev couldn't be reached, or returned a response Blink couldn't
    /// read. No conclusion can be drawn about this project.
    SourceUnavailable { reason: String },
    /// Blink has no OSV ecosystem for this project's language.
    UnsupportedEcosystem { language: String },
    /// There were no packages to audit.
    NothingToAudit,
}

/// A severity rating, as published in the GitHub Advisory Database record
/// that OSV mirrors. Advisories without one (most RUSTSEC-only records, such
/// as "unmaintained" notices) are reported unrated rather than guessed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Moderate,
    High,
    Critical,
}

impl Severity {
    fn parse(raw: &str) -> Option<Self> {
        match raw.to_ascii_uppercase().as_str() {
            "LOW" => Some(Severity::Low),
            "MODERATE" | "MEDIUM" => Some(Severity::Moderate),
            "HIGH" => Some(Severity::High),
            "CRITICAL" => Some(Severity::Critical),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Moderate => "moderate",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }
}

/// One advisory affecting one package. `summary` and `severity` are only
/// populated when the advisory record was successfully fetched from OSV; they
/// are never inferred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Advisory {
    pub id: String,
    /// Other IDs OSV records for the same advisory (e.g. the RUSTSEC ID
    /// behind a GHSA one). Used to avoid counting one advisory twice.
    pub aliases: Vec<String>,
    pub summary: Option<String>,
    pub severity: Option<Severity>,
}

/// A resolved package with one or more known advisories against it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub name: String,
    pub version: String,
    /// True when the project declares this package itself. False means it
    /// arrived through another dependency.
    pub direct: bool,
    /// Shortest chain from a direct dependency to this package, e.g.
    /// `["blink-dashboard", "ratatui", "lru"]`. Empty when the lockfile
    /// format records no edges, in which case no path is claimed.
    pub path: Vec<String>,
    pub advisories: Vec<Advisory>,
}

/// The result of one `blink security` run.
#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    /// The OSV ecosystem queried, e.g. `"crates.io"`.
    pub ecosystem: Option<&'static str>,
    pub scope: AuditScope,
    /// How many distinct `(package, version)` pairs were actually queried.
    pub audited: usize,
    pub status: AuditStatus,
    pub findings: Vec<Finding>,
    /// True when advisory metadata was fetched for every reported ID. False
    /// means some entries carry an ID but no severity or summary.
    pub details_complete: bool,
}

impl AuditReport {
    /// True only when the audit ran to completion and found nothing. A failed
    /// audit is never "clean".
    pub fn is_clean(&self) -> bool {
        self.status == AuditStatus::Completed && self.findings.is_empty()
    }

    /// Every advisory across all findings, deduplicated by ID and alias, so
    /// one advisory affecting two packages is counted once.
    pub fn distinct_advisories(&self) -> Vec<&Advisory> {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut out = Vec::new();
        for advisory in self.findings.iter().flat_map(|f| f.advisories.iter()) {
            if seen.contains(advisory.id.as_str())
                || advisory.aliases.iter().any(|a| seen.contains(a.as_str()))
            {
                continue;
            }
            seen.insert(&advisory.id);
            for alias in &advisory.aliases {
                seen.insert(alias);
            }
            out.push(advisory);
        }
        out
    }

    pub fn direct_findings(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| f.direct)
    }

    pub fn transitive_findings(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| !f.direct)
    }
}

/// A single package to look up: an exact name and version.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuditTarget {
    pub name: String,
    pub version: String,
}

/// Build the set of packages to audit. Prefers the lockfile's fully resolved
/// graph; falls back to the manifest's declared versions when no lockfile for
/// this ecosystem exists. Pure and offline — this is what determines whether
/// the audit covers 7 packages or 251.
pub fn audit_targets(project: &Project, lock: Option<&ResolvedLockfile>) -> Vec<AuditTarget> {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut targets = Vec::new();

    match lock {
        Some(lock) => {
            for pkg in &lock.packages {
                // Workspace members are the project itself, not something
                // published to a registry — there's nothing to look up.
                if pkg.local {
                    continue;
                }
                let key = (pkg.name.clone(), pkg.version.clone());
                if seen.insert(key) {
                    targets.push(AuditTarget {
                        name: pkg.name.clone(),
                        version: pkg.version.clone(),
                    });
                }
            }
        }
        None => {
            for dep in &project.dependencies {
                let version = clean_version(&dep.version);
                let key = (dep.name.clone(), version.clone());
                if seen.insert(key) {
                    targets.push(AuditTarget {
                        name: dep.name.clone(),
                        version,
                    });
                }
            }
        }
    }

    targets
}

/// Name-keyed adjacency over a lockfile's recorded edges, plus the set of
/// names the project depends on directly. Built once and reused for every
/// finding's path lookup.
struct DependencyPaths<'a> {
    edges: HashMap<&'a str, Vec<&'a str>>,
    direct: HashSet<&'a str>,
    records_edges: bool,
}

impl<'a> DependencyPaths<'a> {
    fn new(project: &'a Project, lock: Option<&'a ResolvedLockfile>) -> Self {
        let mut edges: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut direct: HashSet<&str> = HashSet::new();
        let mut records_edges = false;

        // Manifest-declared names are direct by definition, whatever the
        // lockfile says.
        for dep in &project.dependencies {
            direct.insert(dep.name.as_str());
        }

        if let Some(lock) = lock {
            records_edges = lock.records_edges;
            for name in &lock.direct {
                direct.insert(name.as_str());
            }
            for pkg in &lock.packages {
                let entry = edges.entry(pkg.name.as_str()).or_default();
                for dep in &pkg.dependencies {
                    if !entry.contains(&dep.as_str()) {
                        entry.push(dep.as_str());
                    }
                }
            }
        }

        Self {
            edges,
            direct,
            records_edges,
        }
    }

    fn is_direct(&self, name: &str) -> bool {
        self.direct.contains(name)
    }

    /// Shortest chain from any direct dependency to `target`, inclusive at
    /// both ends. Returns an empty path when the lockfile records no edges,
    /// or when no chain exists — Blink shows no path rather than a guessed
    /// one.
    fn path_to(&self, target: &str) -> Vec<String> {
        if !self.records_edges {
            return Vec::new();
        }
        if self.is_direct(target) {
            return vec![target.to_string()];
        }

        let mut previous: HashMap<&str, &str> = HashMap::new();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();

        // Multi-source BFS: start from every direct dependency at once, so
        // the first time `target` is reached it's via a shortest chain.
        let mut roots: Vec<&str> = self.direct.iter().copied().collect();
        roots.sort_unstable();
        for root in roots {
            if visited.insert(root) {
                queue.push_back(root);
            }
        }

        while let Some(current) = queue.pop_front() {
            for next in self.edges.get(current).into_iter().flatten() {
                if !visited.insert(next) {
                    continue;
                }
                previous.insert(next, current);
                if *next == target {
                    let mut path = vec![target];
                    let mut node = target;
                    while let Some(parent) = previous.get(node) {
                        path.push(parent);
                        node = parent;
                    }
                    path.reverse();
                    return path.into_iter().map(str::to_string).collect();
                }
                queue.push_back(next);
            }
        }

        Vec::new()
    }
}

/// Turn raw `(target, advisory ids)` hits into classified findings: direct vs.
/// transitive, with a dependency path where the lockfile supports one. Pure
/// and offline.
pub fn classify_findings(
    project: &Project,
    lock: Option<&ResolvedLockfile>,
    hits: Vec<(AuditTarget, Vec<Advisory>)>,
) -> Vec<Finding> {
    let paths = DependencyPaths::new(project, lock);

    let mut findings: Vec<Finding> = hits
        .into_iter()
        .filter(|(_, advisories)| !advisories.is_empty())
        .map(|(target, advisories)| {
            let direct = paths.is_direct(&target.name);
            let path = paths.path_to(&target.name);
            Finding {
                direct,
                path,
                advisories: dedupe_advisories(advisories),
                name: target.name,
                version: target.version,
            }
        })
        .collect();

    // Worst first, then direct before transitive, then alphabetically — a
    // stable order so output and JSON don't shuffle between runs.
    findings.sort_by(|a, b| {
        worst_severity(b)
            .cmp(&worst_severity(a))
            .then(b.direct.cmp(&a.direct))
            .then(a.name.cmp(&b.name))
            .then(a.version.cmp(&b.version))
    });
    findings
}

fn worst_severity(finding: &Finding) -> Option<Severity> {
    finding.advisories.iter().filter_map(|a| a.severity).max()
}

/// Collapse advisories that OSV records under more than one ID (a GHSA record
/// and its RUSTSEC alias describe one advisory, not two). Records carrying a
/// severity are preferred as the surviving entry.
pub fn dedupe_advisories(mut advisories: Vec<Advisory>) -> Vec<Advisory> {
    advisories.sort_by(|a, b| {
        b.severity
            .is_some()
            .cmp(&a.severity.is_some())
            .then(b.summary.is_some().cmp(&a.summary.is_some()))
            .then(a.id.cmp(&b.id))
    });

    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<Advisory> = Vec::new();
    for advisory in advisories {
        if seen.contains(&advisory.id) || advisory.aliases.iter().any(|a| seen.contains(a)) {
            continue;
        }
        seen.insert(advisory.id.clone());
        for alias in &advisory.aliases {
            seen.insert(alias.clone());
        }
        out.push(advisory);
    }

    // Worst first, so the most serious advisory against a package is the
    // first thing printed for it.
    out.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.id.cmp(&b.id)));
    out
}

// --- OSV.dev wire types -----------------------------------------------------

#[derive(Serialize)]
struct OsvBatchRequest {
    queries: Vec<OsvQuery>,
}

#[derive(Serialize, Clone)]
struct OsvQuery {
    package: OsvPackage,
    version: String,
}

#[derive(Serialize, Clone)]
struct OsvPackage {
    name: String,
    ecosystem: &'static str,
}

#[derive(Deserialize)]
struct OsvBatchResponse {
    #[serde(default)]
    results: Vec<OsvResult>,
}

#[derive(Deserialize, Default)]
struct OsvResult {
    #[serde(default)]
    vulns: Vec<OsvVulnId>,
}

#[derive(Deserialize)]
struct OsvVulnId {
    id: String,
}

#[derive(Deserialize)]
struct OsvVuln {
    id: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    database_specific: Option<OsvDatabaseSpecific>,
}

#[derive(Deserialize)]
struct OsvDatabaseSpecific {
    #[serde(default)]
    severity: Option<String>,
}

// --- The audit --------------------------------------------------------------

/// Audit a project's **fully resolved** dependency graph against
/// [OSV.dev](https://osv.dev) (Google's open, free Open Source Vulnerabilities
/// database).
///
/// When a lockfile for the project's ecosystem is present, every package it
/// resolved is queried — transitive dependencies included, which is where the
/// overwhelming majority of real advisories live. Without a lockfile the audit
/// falls back to declared manifest versions and says so; it never silently
/// implies coverage it doesn't have.
///
/// Requires network access. This is separate from and in addition to the
/// `--online` outdated-package check, and is likewise opt-in — nothing in
/// Blink calls it unless you ask for it, so `cargo test` never needs a
/// network.
pub fn audit(project: &Project) -> AuditReport {
    let Some((ecosystem, lockfiles)) = ecosystem_for(project.language) else {
        return AuditReport {
            ecosystem: None,
            scope: AuditScope::DeclaredOnly,
            audited: 0,
            status: AuditStatus::UnsupportedEcosystem {
                language: project.language.to_string(),
            },
            findings: Vec::new(),
            details_complete: true,
        };
    };

    let lock = load_lockfile_from(&project.root, lockfiles);
    let scope = match &lock {
        Some(lock) => AuditScope::Lockfile {
            file: lock.file,
            records_edges: lock.records_edges,
        },
        None => AuditScope::DeclaredOnly,
    };

    let targets = audit_targets(project, lock.as_ref());
    if targets.is_empty() {
        return AuditReport {
            ecosystem: Some(ecosystem),
            scope,
            audited: 0,
            status: AuditStatus::NothingToAudit,
            findings: Vec::new(),
            details_complete: true,
        };
    }

    let ids = match query_osv(ecosystem, &targets) {
        Ok(ids) => ids,
        Err(reason) => {
            return AuditReport {
                ecosystem: Some(ecosystem),
                scope,
                audited: targets.len(),
                status: AuditStatus::SourceUnavailable { reason },
                findings: Vec::new(),
                details_complete: false,
            };
        }
    };

    let unique_ids: Vec<String> = {
        let mut seen = HashSet::new();
        ids.iter()
            .flat_map(|(_, ids)| ids.iter())
            .filter(|id| seen.insert((*id).clone()))
            .cloned()
            .collect()
    };
    let (details, details_complete) = fetch_advisory_details(&unique_ids);

    let hits: Vec<(AuditTarget, Vec<Advisory>)> = ids
        .into_iter()
        .map(|(target, ids)| {
            let advisories = ids
                .into_iter()
                .map(|id| {
                    details.get(&id).cloned().unwrap_or(Advisory {
                        id,
                        aliases: Vec::new(),
                        summary: None,
                        severity: None,
                    })
                })
                .collect();
            (target, advisories)
        })
        .collect();

    AuditReport {
        ecosystem: Some(ecosystem),
        scope,
        audited: targets.len(),
        status: AuditStatus::Completed,
        findings: classify_findings(project, lock.as_ref(), hits),
        details_complete,
    }
}

/// Ask OSV.dev which of `targets` have advisories. Returns the raw advisory
/// IDs per target, or an error string describing why the lookup couldn't be
/// completed — never an empty result standing in for a failure.
fn query_osv(
    ecosystem: &'static str,
    targets: &[AuditTarget],
) -> Result<Vec<(AuditTarget, Vec<String>)>, String> {
    let mut hits = Vec::new();

    for chunk in targets.chunks(BATCH_SIZE) {
        let queries: Vec<OsvQuery> = chunk
            .iter()
            .map(|target| OsvQuery {
                package: OsvPackage {
                    name: target.name.clone(),
                    ecosystem,
                },
                version: target.version.clone(),
            })
            .collect();

        let response = ureq::post("https://api.osv.dev/v1/querybatch")
            .send_json(OsvBatchRequest { queries })
            .map_err(|err| format!("OSV.dev request failed: {err}"))?
            .into_json::<OsvBatchResponse>()
            .map_err(|err| format!("OSV.dev returned an unreadable response: {err}"))?;

        if response.results.len() != chunk.len() {
            return Err(format!(
                "OSV.dev returned {} results for {} queried packages",
                response.results.len(),
                chunk.len()
            ));
        }

        for (target, result) in chunk.iter().zip(response.results) {
            if result.vulns.is_empty() {
                continue;
            }
            hits.push((
                target.clone(),
                result.vulns.into_iter().map(|v| v.id).collect(),
            ));
        }
    }

    Ok(hits)
}

/// Fetch each advisory record so findings can carry a real severity, summary,
/// and alias list. Returns the details it managed to fetch plus whether the
/// set is complete; an ID whose record couldn't be fetched is still reported,
/// just without metadata.
fn fetch_advisory_details(ids: &[String]) -> (HashMap<String, Advisory>, bool) {
    let mut details = HashMap::new();
    let mut complete = true;

    for id in ids {
        let fetched = ureq::get(&format!("https://api.osv.dev/v1/vulns/{id}"))
            .call()
            .ok()
            .and_then(|r| r.into_json::<OsvVuln>().ok());

        match fetched {
            Some(vuln) => {
                let severity = vuln
                    .database_specific
                    .and_then(|d| d.severity)
                    .as_deref()
                    .and_then(Severity::parse);
                details.insert(
                    id.clone(),
                    Advisory {
                        id: vuln.id,
                        aliases: vuln.aliases,
                        summary: vuln.summary,
                        severity,
                    },
                );
            }
            None => complete = false,
        }
    }

    (details, complete)
}

// --- Back-compatible surface used by `recommend` / `ci` ---------------------

/// A dependency with one or more known vulnerabilities reported by OSV.dev.
#[derive(Debug, Clone, Serialize)]
pub struct VulnerablePackage {
    pub name: String,
    pub version: String,
    /// OSV vulnerability IDs (e.g. `GHSA-...`, `RUSTSEC-...`).
    pub ids: Vec<String>,
}

/// Run [`audit`] and flatten it to the simple package list the recommendation
/// engine consumes. Returns `None` when the audit couldn't be completed, so
/// callers report "unknown" instead of a false "clean".
pub fn find_vulnerabilities(project: &Project) -> Option<Vec<VulnerablePackage>> {
    let report = audit(project);
    if report.status != AuditStatus::Completed {
        return None;
    }
    Some(
        report
            .findings
            .into_iter()
            .map(|finding| VulnerablePackage {
                name: finding.name,
                version: finding.version,
                ids: finding.advisories.into_iter().map(|a| a.id).collect(),
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::parse_lockfile;
    use blink_core::{Dependency, Framework, PackageManager};
    use std::path::PathBuf;

    /// A `Cargo.lock` shaped like the real thing: two workspace members
    /// (no `source`), a direct dependency, and a vulnerable package reachable
    /// only through it.
    const CARGO_LOCK: &str = r#"
[[package]]
name = "demo-cli"
version = "0.1.0"
dependencies = ["demo-core", "ratatui"]

[[package]]
name = "demo-core"
version = "0.1.0"
dependencies = ["serde"]

[[package]]
name = "ratatui"
version = "0.29.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
dependencies = ["lru 0.12.5"]

[[package]]
name = "lru"
version = "0.12.5"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "serde"
version = "1.0.190"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    fn project(deps: &[(&str, &str)]) -> Project {
        Project {
            name: "demo".to_string(),
            root: PathBuf::from("."),
            language: Language::Rust,
            framework: Framework::Cargo,
            package_manager: PackageManager::Cargo,
            dependencies: deps
                .iter()
                .map(|(name, version)| Dependency {
                    name: (*name).to_string(),
                    version: (*version).to_string(),
                    dev: false,
                })
                .collect(),
            file_count: 0,
            config_file: "Cargo.toml".to_string(),
            is_workspace: true,
            has_virtualenv: false,
        }
    }

    fn advisory(id: &str, aliases: &[&str], severity: Option<Severity>) -> Advisory {
        Advisory {
            id: id.to_string(),
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
            summary: None,
            severity,
        }
    }

    #[test]
    fn audits_the_whole_lockfile_not_just_declared_dependencies() {
        // The manifest declares one dependency; the lockfile resolves three
        // third-party packages. The audit must cover all three.
        let project = project(&[("ratatui", "0.29")]);
        let lock = parse_lockfile("Cargo.lock", CARGO_LOCK);

        let declared_only = audit_targets(&project, None);
        assert_eq!(declared_only.len(), 1);

        let full = audit_targets(&project, Some(&lock));
        let names: Vec<&str> = full.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["ratatui", "lru", "serde"]);
        // Workspace members are the project itself and are never queried.
        assert!(!names.contains(&"demo-cli"));
        assert!(!names.contains(&"demo-core"));
        // Versions come from the lockfile, not the manifest range.
        assert_eq!(full[0].version, "0.29.0");
    }

    #[test]
    fn falls_back_to_declared_versions_without_a_lockfile() {
        let project = project(&[("ratatui", "^0.29.0")]);
        let targets = audit_targets(&project, None);
        assert_eq!(targets.len(), 1);
        // The semver range operator is stripped so OSV gets a plain version.
        assert_eq!(targets[0].version, "0.29.0");
    }

    #[test]
    fn classifies_transitive_findings_and_shows_the_path() {
        let project = project(&[]);
        let lock = parse_lockfile("Cargo.lock", CARGO_LOCK);
        let hits = vec![(
            AuditTarget {
                name: "lru".to_string(),
                version: "0.12.5".to_string(),
            },
            vec![advisory("RUSTSEC-2026-0002", &[], None)],
        )];

        let findings = classify_findings(&project, Some(&lock), hits);
        assert_eq!(findings.len(), 1);
        assert!(!findings[0].direct);
        assert_eq!(
            findings[0].path,
            vec!["ratatui".to_string(), "lru".to_string()]
        );
    }

    #[test]
    fn classifies_direct_findings() {
        let project = project(&[]);
        let lock = parse_lockfile("Cargo.lock", CARGO_LOCK);
        let hits = vec![(
            AuditTarget {
                name: "ratatui".to_string(),
                version: "0.29.0".to_string(),
            },
            vec![advisory("GHSA-xxxx", &[], Some(Severity::High))],
        )];

        let findings = classify_findings(&project, Some(&lock), hits);
        assert!(findings[0].direct);
        assert_eq!(findings[0].path, vec!["ratatui".to_string()]);
    }

    #[test]
    fn claims_no_path_when_the_lockfile_records_no_edges() {
        let project = project(&[]);
        let lock = parse_lockfile(
            "yarn.lock",
            "react@^18.2.0:\n  version \"18.2.0\"\n\nlru@^1.0.0:\n  version \"1.0.0\"\n",
        );
        let hits = vec![(
            AuditTarget {
                name: "lru".to_string(),
                version: "1.0.0".to_string(),
            },
            vec![advisory("GHSA-xxxx", &[], None)],
        )];

        let findings = classify_findings(&project, Some(&lock), hits);
        assert!(findings[0].path.is_empty());
        assert!(!findings[0].direct);
    }

    #[test]
    fn collapses_aliased_advisory_ids_into_one() {
        // OSV returns both the GHSA record and its RUSTSEC alias for the same
        // advisory; counting them separately would overstate the finding.
        let deduped = dedupe_advisories(vec![
            advisory("RUSTSEC-2026-0002", &["GHSA-rhfx"], None),
            advisory("GHSA-rhfx", &["RUSTSEC-2026-0002"], Some(Severity::Low)),
        ]);

        assert_eq!(deduped.len(), 1);
        // The record carrying a severity survives.
        assert_eq!(deduped[0].id, "GHSA-rhfx");
        assert_eq!(deduped[0].severity, Some(Severity::Low));
    }

    #[test]
    fn keeps_unrelated_advisories_separate() {
        let deduped = dedupe_advisories(vec![
            advisory("RUSTSEC-2024-0436", &[], None),
            advisory("RUSTSEC-2025-0119", &[], None),
        ]);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn sorts_findings_worst_first() {
        let project = project(&[]);
        let hits = vec![
            (
                AuditTarget {
                    name: "aaa".to_string(),
                    version: "1.0.0".to_string(),
                },
                vec![advisory("GHSA-low", &[], Some(Severity::Low))],
            ),
            (
                AuditTarget {
                    name: "zzz".to_string(),
                    version: "1.0.0".to_string(),
                },
                vec![advisory("GHSA-high", &[], Some(Severity::High))],
            ),
        ];

        let findings = classify_findings(&project, None, hits);
        assert_eq!(findings[0].name, "zzz");
        assert_eq!(findings[1].name, "aaa");
    }

    #[test]
    fn an_unreachable_source_is_never_reported_as_clean() {
        let report = AuditReport {
            ecosystem: Some("crates.io"),
            scope: AuditScope::Lockfile {
                file: "Cargo.lock",
                records_edges: true,
            },
            audited: 251,
            status: AuditStatus::SourceUnavailable {
                reason: "offline".to_string(),
            },
            findings: Vec::new(),
            details_complete: false,
        };
        assert!(!report.is_clean());

        let completed = AuditReport {
            status: AuditStatus::Completed,
            ..report
        };
        assert!(completed.is_clean());
    }

    #[test]
    fn counts_one_advisory_affecting_two_packages_once() {
        let project = project(&[]);
        let hits = vec![
            (
                AuditTarget {
                    name: "aaa".to_string(),
                    version: "1.0.0".to_string(),
                },
                vec![advisory("GHSA-shared", &[], Some(Severity::High))],
            ),
            (
                AuditTarget {
                    name: "bbb".to_string(),
                    version: "1.0.0".to_string(),
                },
                vec![advisory("GHSA-shared", &[], Some(Severity::High))],
            ),
        ];
        let report = AuditReport {
            ecosystem: Some("crates.io"),
            scope: AuditScope::DeclaredOnly,
            audited: 2,
            status: AuditStatus::Completed,
            findings: classify_findings(&project, None, hits),
            details_complete: true,
        };

        assert_eq!(report.findings.len(), 2);
        assert_eq!(report.distinct_advisories().len(), 1);
    }

    #[test]
    fn parses_only_known_severity_labels() {
        assert_eq!(Severity::parse("HIGH"), Some(Severity::High));
        assert_eq!(Severity::parse("moderate"), Some(Severity::Moderate));
        assert_eq!(Severity::parse("Medium"), Some(Severity::Moderate));
        // Anything unrecognized stays unrated rather than being guessed.
        assert_eq!(Severity::parse("SEVERE"), None);
        assert_eq!(Severity::parse(""), None);
    }
}

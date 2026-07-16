use blink_core::{Language, Project};
use serde::{Deserialize, Serialize};

use crate::version::clean_version;

/// A dependency with one or more known vulnerabilities reported by OSV.dev.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VulnerablePackage {
    pub name: String,
    pub version: String,
    /// OSV vulnerability IDs (e.g. `GHSA-...`, `RUSTSEC-...`). Blink shows
    /// the IDs so you can look them up; it doesn't fetch or summarize the
    /// underlying advisories.
    pub ids: Vec<String>,
}

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

/// Query [OSV.dev](https://osv.dev) (Google's open, free Open Source
/// Vulnerabilities database) for each dependency's declared version.
/// Requires network access; this is separate from and in addition to the
/// `--online` outdated-package check, and is likewise opt-in.
pub fn find_vulnerabilities(project: &Project) -> Vec<VulnerablePackage> {
    let ecosystem = match project.language {
        Language::Rust => "crates.io",
        Language::TypeScript | Language::JavaScript => "npm",
        Language::Python => "PyPI",
        Language::Unknown => return Vec::new(),
    };

    if project.dependencies.is_empty() {
        return Vec::new();
    }

    let queries: Vec<OsvQuery> = project
        .dependencies
        .iter()
        .map(|dep| OsvQuery {
            package: OsvPackage {
                name: dep.name.clone(),
                ecosystem,
            },
            version: clean_version(&dep.version),
        })
        .collect();

    let response = ureq::post("https://api.osv.dev/v1/querybatch")
        .send_json(OsvBatchRequest { queries })
        .ok()
        .and_then(|r| r.into_json::<OsvBatchResponse>().ok());

    let Some(response) = response else {
        return Vec::new();
    };

    project
        .dependencies
        .iter()
        .zip(response.results.iter())
        .filter_map(|(dep, result)| {
            if result.vulns.is_empty() {
                return None;
            }
            Some(VulnerablePackage {
                name: dep.name.clone(),
                version: dep.version.clone(),
                ids: result.vulns.iter().map(|v| v.id.clone()).collect(),
            })
        })
        .collect()
}

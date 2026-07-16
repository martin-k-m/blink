use blink_core::{Language, Project};
use semver::Version;
use serde::Deserialize;

/// A dependency whose declared version trails the latest release published
/// on its package registry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OutdatedPackage {
    pub name: String,
    pub current: String,
    pub latest: String,
}

#[derive(Deserialize)]
struct CratesIoResponse {
    #[serde(rename = "crate")]
    krate: CratesIoCrate,
}

#[derive(Deserialize)]
struct CratesIoCrate {
    max_stable_version: String,
}

#[derive(Deserialize)]
struct NpmResponse {
    #[serde(rename = "dist-tags")]
    dist_tags: NpmDistTags,
}

#[derive(Deserialize)]
struct NpmDistTags {
    latest: String,
}

/// Query the appropriate public registry (crates.io or the npm registry)
/// for each dependency's latest published version and compare it against
/// what the project has declared. Requires network access; callers opt in
/// explicitly (e.g. via a `--online` CLI flag) since this is the only part
/// of Blink's analyzer that leaves the machine.
pub fn find_outdated(project: &Project) -> Vec<OutdatedPackage> {
    let checker: fn(&str) -> Option<String> = match project.language {
        Language::Rust => latest_crates_io_version,
        Language::TypeScript | Language::JavaScript => latest_npm_version,
        Language::Python | Language::Unknown => return Vec::new(),
    };

    project
        .dependencies
        .iter()
        .filter_map(|dep| {
            let latest = checker(&dep.name)?;
            let current = clean_version(&dep.version);
            let is_outdated = match (Version::parse(&current), Version::parse(&latest)) {
                (Ok(current_v), Ok(latest_v)) => latest_v > current_v,
                _ => current != latest,
            };
            is_outdated.then_some(OutdatedPackage {
                name: dep.name.clone(),
                current: dep.version.clone(),
                latest,
            })
        })
        .collect()
}

fn clean_version(raw: &str) -> String {
    raw.trim_start_matches(['^', '~', '=', '>', '<', ' '])
        .to_string()
}

fn latest_crates_io_version(name: &str) -> Option<String> {
    let url = format!("https://crates.io/api/v1/crates/{name}");
    let response: CratesIoResponse = ureq::get(&url)
        .set("User-Agent", "blink (https://github.com/blink-dev/blink)")
        .call()
        .ok()?
        .into_json()
        .ok()?;
    Some(response.krate.max_stable_version)
}

fn latest_npm_version(name: &str) -> Option<String> {
    let url = format!("https://registry.npmjs.org/{name}");
    let response: NpmResponse = ureq::get(&url).call().ok()?.into_json().ok()?;
    Some(response.dist_tags.latest)
}

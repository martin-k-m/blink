use std::collections::BTreeMap;
use std::path::Path;

/// A package that resolves to more than one distinct version in the
/// project's lockfile.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DuplicatePackage {
    pub name: String,
    pub versions: Vec<String>,
}

/// Find packages resolved at more than one version by reading the project's
/// lockfile (`Cargo.lock` or `package-lock.json`). Returns an empty list if
/// no lockfile is present, since duplicate *resolution* can only be observed
/// once a package manager has actually locked the dependency tree.
pub fn find_duplicates(root: &Path) -> Vec<DuplicatePackage> {
    if root.join("Cargo.lock").is_file() {
        return find_cargo_duplicates(&root.join("Cargo.lock"));
    }
    if root.join("package-lock.json").is_file() {
        return find_npm_duplicates(&root.join("package-lock.json"));
    }
    Vec::new()
}

fn find_cargo_duplicates(path: &Path) -> Vec<DuplicatePackage> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(doc) = raw.parse::<toml::Value>() else {
        return Vec::new();
    };

    let mut versions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if let Some(packages) = doc.get("package").and_then(|p| p.as_array()) {
        for pkg in packages {
            let (Some(name), Some(version)) = (
                pkg.get("name").and_then(|v| v.as_str()),
                pkg.get("version").and_then(|v| v.as_str()),
            ) else {
                continue;
            };
            let entry = versions.entry(name.to_string()).or_default();
            if !entry.iter().any(|v| v == version) {
                entry.push(version.to_string());
            }
        }
    }

    to_duplicates(versions)
}

fn find_npm_duplicates(path: &Path) -> Vec<DuplicatePackage> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };

    let mut versions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if let Some(packages) = doc.get("packages").and_then(|p| p.as_object()) {
        for (key, meta) in packages {
            if key.is_empty() {
                continue; // the root project entry
            }
            let Some(name) = key.rsplit("node_modules/").next() else {
                continue;
            };
            let Some(version) = meta.get("version").and_then(|v| v.as_str()) else {
                continue;
            };
            let entry = versions.entry(name.to_string()).or_default();
            if !entry.iter().any(|v| v == version) {
                entry.push(version.to_string());
            }
        }
    }

    to_duplicates(versions)
}

fn to_duplicates(versions: BTreeMap<String, Vec<String>>) -> Vec<DuplicatePackage> {
    let mut duplicates: Vec<DuplicatePackage> = versions
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(name, mut versions)| {
            versions.sort();
            DuplicatePackage { name, versions }
        })
        .collect();
    duplicates.sort_by(|a, b| a.name.cmp(&b.name));
    duplicates
}

use std::collections::BTreeMap;

use blink_parser::LockedPackage;

/// A package that resolves to more than one distinct version in the
/// project's lockfile.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicatePackage {
    pub name: String,
    pub versions: Vec<String>,
}

/// Find packages resolved at more than one version among `locked`.
pub fn find_duplicates(locked: &[LockedPackage]) -> Vec<DuplicatePackage> {
    let mut versions: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for pkg in locked {
        let entry = versions.entry(pkg.name.as_str()).or_default();
        if !entry.contains(&pkg.version.as_str()) {
            entry.push(&pkg.version);
        }
    }

    let mut duplicates: Vec<DuplicatePackage> = versions
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(name, mut versions)| {
            versions.sort_unstable();
            DuplicatePackage {
                name: name.to_string(),
                versions: versions.into_iter().map(str::to_string).collect(),
            }
        })
        .collect();
    duplicates.sort_by(|a, b| a.name.cmp(&b.name));
    duplicates
}

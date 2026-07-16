/// A package as actually resolved in a lockfile: a name and the exact
/// version the package manager settled on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
}

/// Parse a `Cargo.lock`. Returns one entry per `[[package]]` table, in the
/// order they appear. Malformed TOML yields an empty list rather than an
/// error — a missing/corrupt lockfile just means "nothing resolved yet".
pub fn parse_cargo_lock(raw: &str) -> Vec<LockedPackage> {
    let Ok(doc) = raw.parse::<toml::Value>() else {
        return Vec::new();
    };

    let Some(packages) = doc.get("package").and_then(|p| p.as_array()) else {
        return Vec::new();
    };

    packages
        .iter()
        .filter_map(|pkg| {
            let name = pkg.get("name")?.as_str()?.to_string();
            let version = pkg.get("version")?.as_str()?.to_string();
            Some(LockedPackage { name, version })
        })
        .collect()
}

/// Parse a `package-lock.json` (lockfile v2/v3 format, which uses a flat
/// `packages` map keyed by `node_modules/<name>` path). Returns one entry
/// per resolved package, excluding the root project entry itself.
pub fn parse_npm_lock(raw: &str) -> Vec<LockedPackage> {
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };

    let Some(packages) = doc.get("packages").and_then(|p| p.as_object()) else {
        return Vec::new();
    };

    packages
        .iter()
        .filter_map(|(key, meta)| {
            if key.is_empty() {
                return None; // the root project entry
            }
            let name = key.rsplit("node_modules/").next()?.to_string();
            let version = meta.get("version")?.as_str()?.to_string();
            Some(LockedPackage { name, version })
        })
        .collect()
}

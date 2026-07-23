/// A package as actually resolved in a lockfile: a name, the exact version
/// the package manager settled on, and — for formats that record them — the
/// names of the packages it depends on.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    /// Names of this package's own dependencies, as recorded in the lockfile.
    /// Empty for formats that don't record edges (`yarn.lock`,
    /// `pnpm-lock.yaml`) — an empty list therefore means "unknown", not
    /// "no dependencies".
    pub dependencies: Vec<String>,
    /// True when the lockfile marks this entry as part of the local project
    /// (a Cargo workspace member) rather than a downloaded package.
    pub local: bool,
}

impl LockedPackage {
    /// Convenience constructor for the common "name + version, no recorded
    /// edges" case.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            dependencies: Vec::new(),
            local: false,
        }
    }
}

/// Parse a `Cargo.lock`. Returns one entry per `[[package]]` table, in the
/// order they appear. Malformed TOML yields an empty list rather than an
/// error — a missing/corrupt lockfile just means "nothing resolved yet".
///
/// Entries without a `source` key are workspace members (the local project
/// itself) and are flagged `local`. Each entry's `dependencies` array is
/// recorded name-only: Cargo writes `"name"` or `"name version"` depending on
/// whether the name alone is ambiguous, and only the name is kept here.
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
            let dependencies = pkg
                .get("dependencies")
                .and_then(|d| d.as_array())
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|e| e.as_str())
                        .filter_map(|e| e.split_whitespace().next())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            Some(LockedPackage {
                name,
                version,
                dependencies,
                local: pkg.get("source").is_none(),
            })
        })
        .collect()
}

/// Parse a `package-lock.json` (lockfile v2/v3 format, which uses a flat
/// `packages` map keyed by `node_modules/<name>` path). Returns one entry
/// per resolved package, excluding the root project entry itself (use
/// [`parse_npm_lock_direct`] for the root's own dependency names).
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
            Some(LockedPackage {
                name,
                version,
                dependencies: npm_dependency_names(meta),
                local: false,
            })
        })
        .collect()
}

/// The names a `package-lock.json`'s root entry declares directly, across
/// `dependencies`, `devDependencies`, and `optionalDependencies`. These are
/// the project's *direct* dependencies; everything else in the lockfile
/// arrived transitively.
pub fn parse_npm_lock_direct(raw: &str) -> Vec<String> {
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };

    let Some(root) = doc.get("packages").and_then(|p| p.get("")) else {
        return Vec::new();
    };

    npm_dependency_names(root)
}

fn npm_dependency_names(meta: &serde_json::Value) -> Vec<String> {
    let mut names = Vec::new();
    for field in ["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(map) = meta.get(field).and_then(|d| d.as_object()) {
            for name in map.keys() {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }
    }
    names
}

/// Parse a `yarn.lock`. Handles both the classic v1 format (`version "1.2.3"`)
/// and Yarn Berry's YAML-ish format (`version: 1.2.3`).
///
/// Only the resolved name/version pairs are recovered — neither format
/// records a usable package-to-package edge list here, so `dependencies` is
/// always empty and callers must not read it as "has no dependencies".
pub fn parse_yarn_lock(raw: &str) -> Vec<LockedPackage> {
    let mut packages: Vec<LockedPackage> = Vec::new();
    let mut pending: Vec<String> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if !line.starts_with(char::is_whitespace) {
            // An entry header: one or more comma-separated `name@range`
            // specifiers, optionally quoted, ending in `:`.
            pending = trimmed
                .trim_end_matches(':')
                .split(", ")
                .filter_map(|spec| yarn_spec_name(spec.trim().trim_matches(['"', '\''])))
                .collect();
            continue;
        }

        if let Some(rest) = trimmed
            .strip_prefix("version")
            .filter(|rest| rest.starts_with([':', ' ']))
        {
            let version = rest
                .trim_start_matches([':', ' '])
                .trim()
                .trim_matches(['"', '\'']);
            if version.is_empty() {
                continue;
            }
            for name in pending.drain(..) {
                if !packages
                    .iter()
                    .any(|p| p.name == name && p.version == version)
                {
                    packages.push(LockedPackage::new(name, version));
                }
            }
        }
    }

    packages
}

/// Split a yarn entry specifier (`react@^18.2.0`, `@scope/pkg@npm:1.0.0`)
/// into its package name. Returns `None` when there's no `@`-separated range,
/// since a bare name gives no resolved package to audit.
fn yarn_spec_name(spec: &str) -> Option<String> {
    let (name, _range) = if let Some(rest) = spec.strip_prefix('@') {
        let idx = rest.find('@')?;
        (&spec[..idx + 1], &rest[idx + 1..])
    } else {
        let idx = spec.find('@')?;
        (&spec[..idx], &spec[idx + 1..])
    };
    (!name.is_empty()).then(|| name.to_string())
}

/// Parse a `pnpm-lock.yaml`. pnpm keys its resolved packages by
/// `/<name>/<version>` (lockfile v5), `/<name>@<version>` (v6), or
/// `<name>@<version>` (v9), under a top-level `packages:` (and, in v9, also
/// `snapshots:`) block. Only those keys are read — the rest of the document
/// is left alone, so this needs no YAML dependency.
///
/// Only resolved name/version pairs are recovered. pnpm v9 does record edges
/// under `snapshots:`, but they aren't read here, so `dependencies` is always
/// empty and callers must treat it as "unknown", not "none".
pub fn parse_pnpm_lock(raw: &str) -> Vec<LockedPackage> {
    let mut packages: Vec<LockedPackage> = Vec::new();
    let mut in_packages = false;

    for line in raw.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }

        if !line.starts_with(char::is_whitespace) {
            in_packages = matches!(line.trim_end_matches(':').trim(), "packages" | "snapshots");
            continue;
        }

        if !in_packages {
            continue;
        }

        // Package keys sit exactly one level in ("  key:"); anything deeper is
        // that entry's metadata.
        let indent = line.len() - line.trim_start().len();
        if indent != 2 {
            continue;
        }

        // The key runs up to the first `:`; in v9's `snapshots:` block an
        // inline value follows it (`minimist@0.0.8: {}`) and must not be
        // swallowed into the version.
        let key = line.split(':').next().unwrap_or_default();
        let key = key.trim().trim_matches(['"', '\'']);
        let Some((name, version)) = pnpm_split_key(key) else {
            continue;
        };
        if !packages
            .iter()
            .any(|p| p.name == name && p.version == version)
        {
            packages.push(LockedPackage::new(name, version));
        }
    }

    packages
}

/// Split a pnpm package key into `(name, version)`. Peer-dependency suffixes
/// (`(react@18.2.0)`) and v5's leading `/` are stripped. Returns `None` when
/// the key doesn't carry an exact version.
fn pnpm_split_key(key: &str) -> Option<(String, String)> {
    let key = key.split('(').next()?.trim();
    let key = key.strip_prefix('/').unwrap_or(key);

    // v5 uses `/name/version`; v6/v9 use `name@version`. Prefer the `@`
    // split, falling back to the last `/` segment.
    let (name, version) = match key.rfind('@') {
        Some(idx) if idx > 0 => (&key[..idx], &key[idx + 1..]),
        _ => {
            let idx = key.rfind('/')?;
            (&key[..idx], &key[idx + 1..])
        }
    };

    let name = name.trim_end_matches('/');
    if name.is_empty() || version.is_empty() {
        return None;
    }
    // A version always starts with a digit; this rejects keys like plain
    // `@scope/name` that carry no resolved version.
    if !version.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    Some((name.to_string(), version.to_string()))
}

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::dependency::RawDependency;
use crate::error::Result;

/// The subset of `Cargo.toml` Blink reads: the package name, its two
/// dependency tables, and whether it declares a `[workspace]`.
#[derive(Debug, Clone, Default)]
pub struct CargoManifest {
    pub name: Option<String>,
    pub dependencies: Vec<RawDependency>,
    pub is_workspace: bool,
}

#[derive(Debug, Deserialize, Default)]
struct CargoManifestRaw {
    #[serde(default)]
    package: Option<CargoPackageRaw>,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
    #[serde(default)]
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: BTreeMap<String, toml::Value>,
    #[serde(default)]
    workspace: Option<toml::Value>,
}

#[derive(Debug, Deserialize, Default)]
struct CargoPackageRaw {
    name: Option<String>,
}

/// Parse a `Cargo.toml`. Malformed TOML or a manifest missing the fields
/// Blink cares about both fall back to an empty manifest rather than
/// erroring — a package with no declared dependencies is a valid manifest.
pub fn parse_cargo_manifest(raw: &str) -> CargoManifest {
    let parsed: CargoManifestRaw = toml::from_str(raw).unwrap_or_default();

    let mut dependencies: Vec<RawDependency> = parsed
        .dependencies
        .into_iter()
        .map(|(name, value)| RawDependency {
            name,
            version: cargo_dep_version(&value),
            dev: false,
        })
        .collect();
    dependencies.extend(
        parsed
            .dev_dependencies
            .into_iter()
            .map(|(name, value)| RawDependency {
                name,
                version: cargo_dep_version(&value),
                dev: true,
            }),
    );
    dependencies.sort_by(|a, b| a.name.cmp(&b.name));

    CargoManifest {
        name: parsed.package.and_then(|p| p.name),
        dependencies,
        is_workspace: parsed.workspace.is_some(),
    }
}

fn cargo_dep_version(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string(),
        _ => "*".to_string(),
    }
}

/// The subset of `package.json` Blink reads.
#[derive(Debug, Clone, Default)]
pub struct PackageJsonManifest {
    pub name: Option<String>,
    pub dependencies: Vec<RawDependency>,
}

impl PackageJsonManifest {
    /// Whether `name` appears in either the runtime or dev dependency list.
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.iter().any(|d| d.name == name)
    }
}

#[derive(Debug, Deserialize, Default)]
struct PackageJsonRaw {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(default)]
    #[serde(rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, String>,
}

/// Parse a `package.json`. Unlike `Cargo.toml`, malformed JSON is treated as
/// an error: JSON has no tolerance for stray syntax the way sparse TOML
/// tables do, so if it doesn't parse the file is genuinely broken.
pub fn parse_package_json(raw: &str) -> Result<PackageJsonManifest> {
    let parsed: PackageJsonRaw = serde_json::from_str(raw)?;

    let mut dependencies: Vec<RawDependency> = parsed
        .dependencies
        .into_iter()
        .map(|(name, version)| RawDependency {
            name,
            version,
            dev: false,
        })
        .collect();
    dependencies.extend(
        parsed
            .dev_dependencies
            .into_iter()
            .map(|(name, version)| RawDependency {
                name,
                version,
                dev: true,
            }),
    );
    dependencies.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(PackageJsonManifest {
        name: parsed.name,
        dependencies,
    })
}

/// Parse a `requirements.txt`, one dependency per non-comment line.
/// Requirements files have no dev/runtime split, so everything is `dev: false`.
pub fn parse_requirements_txt(raw: &str) -> Vec<RawDependency> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (name, version) = line
                .split_once("==")
                .or_else(|| line.split_once(">="))
                .or_else(|| line.split_once("~="))
                .unwrap_or((line, "*"));
            RawDependency {
                name: name.trim().to_string(),
                version: version.trim().to_string(),
                dev: false,
            }
        })
        .collect()
}

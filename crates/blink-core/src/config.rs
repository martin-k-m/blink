use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{BlinkError, Result};

/// The conventional name of Blink's configuration file.
pub const CONFIG_FILE_NAME: &str = "blink.toml";

/// The signature Blink config filename. `.bnk` is an *alternate name for the
/// exact same schema* as [`CONFIG_FILE_NAME`] — not a second, parallel format.
/// A directory may use either name; if both are present, `.bnk` wins. Seeing a
/// `.bnk` in a repository is meant to immediately signal "this project uses
/// Blink," while the parser, defaults, and API stay identical to `blink.toml`.
pub const SIGNATURE_CONFIG_FILE_NAME: &str = ".bnk";

/// Accepted config filenames, in resolution order (first match wins).
pub const CONFIG_FILE_NAMES: &[&str] = &[SIGNATURE_CONFIG_FILE_NAME, CONFIG_FILE_NAME];

/// Top-level Blink configuration, read from `.bnk` or `blink.toml`.
///
/// Every table except `[project]` is optional and defaults sensibly, so a
/// minimal config is just a name — and a project with no config file at all
/// still works, using these same defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlinkConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub optimization: OptimizationConfig,
    /// Scan/ignore tuning. `[scan].ignore` is merged with `[project].ignore`;
    /// both exist because the Phase 8 config shape uses `[scan]` while v0.1
    /// shipped `[project].ignore`. See [`BlinkConfig::extra_ignores`].
    #[serde(default)]
    pub scan: ScanConfig,
    /// Named project commands, e.g. `dev = "npm run dev"`. Surfaced by
    /// `blink tasks` and runnable via `blink task <name>`.
    #[serde(default)]
    pub commands: BTreeMap<String, String>,
    #[serde(default)]
    pub index: IndexConfig,
    /// Context-engine tuning: whether context commands are enabled and which
    /// path roots the context graph covers. See [`ContextConfig`].
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub report: ReportConfig,
    /// Named groups of commands run in sequence via `blink profile <name>`.
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileConfig>,
    /// Free-form per-plugin configuration sections, e.g. `[plugins.react]`.
    /// Blink core never interprets these; they're preserved for plugins.
    #[serde(default)]
    pub plugins: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub name: String,
    /// Optional free-form project type label (e.g. `"web"`, `"cli"`), shown by
    /// `blink inspect`. Purely descriptive; Blink does not act on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Extra directory names to skip during scans/builds, in addition to
    /// Blink's built-in ignore list (`.git`, `node_modules`, `target`, ...).
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerConfig {
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 3000 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct OptimizationConfig {
    pub cache: bool,
    pub analyze: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            cache: true,
            analyze: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ScanConfig {
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct IndexConfig {
    /// Whether commands may build/use the on-disk index.
    pub enabled: bool,
    /// Whether the index refreshes automatically before commands that use it.
    pub auto_update: bool,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_update: true,
        }
    }
}

/// `[context]` — configures Blink's context engine (`context`/`query`/
/// `explain`/`map`/`export`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ContextConfig {
    /// Whether context commands may run. Defaults true.
    pub enabled: bool,
    /// Path roots (project-relative) the context graph is limited to. Empty
    /// (the default) means the whole project. A root matches on a path-segment
    /// boundary, so `"src"` covers `src/main.rs` but not `srcgen/x.rs`.
    pub include: Vec<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ReportConfig {
    /// Preferred `blink report` format when none is given on the CLI:
    /// `"json"`, `"markdown"`, or `"html"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileConfig {
    #[serde(default)]
    pub commands: Vec<String>,
}

impl BlinkConfig {
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            project: ProjectConfig {
                name: project_name.into(),
                r#type: None,
                ignore: Vec::new(),
            },
            server: ServerConfig::default(),
            optimization: OptimizationConfig::default(),
            scan: ScanConfig::default(),
            commands: BTreeMap::new(),
            index: IndexConfig::default(),
            context: ContextConfig::default(),
            report: ReportConfig::default(),
            profiles: BTreeMap::new(),
            plugins: BTreeMap::new(),
        }
    }

    /// The config file Blink would read for `dir`, if any: `.bnk` preferred,
    /// then `blink.toml`. Returns `None` when neither exists.
    pub fn config_path(dir: &Path) -> Option<PathBuf> {
        CONFIG_FILE_NAMES
            .iter()
            .map(|name| dir.join(name))
            .find(|path| path.is_file())
    }

    /// Load configuration from `dir`, reading `.bnk` or `blink.toml`.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = Self::config_path(dir).ok_or_else(|| BlinkError::Io {
            path: dir.join(CONFIG_FILE_NAME),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no .bnk or blink.toml found",
            ),
        })?;
        let raw = std::fs::read_to_string(&path).map_err(|source| BlinkError::Io {
            path: path.clone(),
            source,
        })?;
        toml::from_str(&raw).map_err(|source| BlinkError::ConfigParse {
            path,
            source: Box::new(source),
        })
    }

    /// Serialize this configuration to TOML.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(BlinkError::from)
    }

    /// Write this configuration to `dir`/blink.toml.
    pub fn write(&self, dir: &Path) -> Result<()> {
        self.write_as(dir, CONFIG_FILE_NAME)
    }

    /// Write this configuration to `dir`/`filename` (e.g. `.bnk`).
    pub fn write_as(&self, dir: &Path, filename: &str) -> Result<()> {
        let path = dir.join(filename);
        let contents = self.to_toml()?;
        std::fs::write(&path, contents).map_err(|source| BlinkError::Io { path, source })
    }

    /// Whether a config file (`.bnk` or `blink.toml`) exists in `dir`.
    pub fn exists(dir: &Path) -> bool {
        Self::config_path(dir).is_some()
    }

    /// All extra ignore directory names, merging `[project].ignore` and
    /// `[scan].ignore` (deduplicated, order preserved).
    pub fn extra_ignores(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for entry in self.project.ignore.iter().chain(self.scan.ignore.iter()) {
            if !out.contains(entry) {
                out.push(entry.clone());
            }
        }
        out
    }
}

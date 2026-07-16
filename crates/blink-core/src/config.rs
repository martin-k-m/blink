use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{BlinkError, Result};

/// The name of Blink's configuration file, expected at a project's root.
pub const CONFIG_FILE_NAME: &str = "blink.toml";

/// Top-level `blink.toml` configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlinkConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub name: String,
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

impl BlinkConfig {
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            project: ProjectConfig {
                name: project_name.into(),
                ignore: Vec::new(),
            },
            server: ServerConfig::default(),
            optimization: OptimizationConfig::default(),
        }
    }

    /// Load configuration from `dir`/blink.toml.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(CONFIG_FILE_NAME);
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
        let path = dir.join(CONFIG_FILE_NAME);
        let contents = self.to_toml()?;
        std::fs::write(&path, contents).map_err(|source| BlinkError::Io { path, source })
    }

    pub fn exists(dir: &Path) -> bool {
        dir.join(CONFIG_FILE_NAME).is_file()
    }
}

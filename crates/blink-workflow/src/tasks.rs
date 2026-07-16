//! Task discovery across the conventions Blink understands: `package.json`
//! scripts, `Makefile` targets, `justfile` recipes, Cargo aliases, and
//! `[commands]` in Blink's own config. Discovery only *reads* — running a task
//! is the caller's job (`blink task <name>`).

use std::fmt;
use std::path::Path;

use blink_core::BlinkConfig;

/// Where a discovered task came from — shown so users know which underlying
/// tool actually runs, keeping Blink transparent rather than a black box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSource {
    BlinkConfig,
    PackageJson,
    Makefile,
    Justfile,
    CargoAlias,
}

impl fmt::Display for TaskSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskSource::BlinkConfig => "blink.toml",
            TaskSource::PackageJson => "package.json",
            TaskSource::Makefile => "Makefile",
            TaskSource::Justfile => "justfile",
            TaskSource::CargoAlias => "Cargo alias",
        };
        write!(f, "{s}")
    }
}

/// A runnable task: its name, the shell command it maps to, and its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub name: String,
    pub command: String,
    pub source: TaskSource,
}

/// Discover every task defined for the project at `root`.
///
/// Config `[commands]` are listed first and take precedence: if the same name
/// appears in both `blink.toml` and, say, `package.json`, the config entry
/// wins (Blink lets a project override what a task means). Otherwise order is
/// config → package.json → Makefile → justfile → Cargo aliases.
pub fn discover(root: &Path) -> Vec<Task> {
    let mut out: Vec<Task> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    if let Ok(config) = BlinkConfig::load(root) {
        for (name, command) in config.commands {
            add(&mut out, &mut names, name, command, TaskSource::BlinkConfig);
        }
    }
    for (name, command) in package_json_scripts(root) {
        add(&mut out, &mut names, name, command, TaskSource::PackageJson);
    }
    for name in makefile_targets(root) {
        let command = format!("make {name}");
        add(&mut out, &mut names, name, command, TaskSource::Makefile);
    }
    for name in justfile_recipes(root) {
        let command = format!("just {name}");
        add(&mut out, &mut names, name, command, TaskSource::Justfile);
    }
    for name in cargo_aliases(root) {
        let command = format!("cargo {name}");
        add(&mut out, &mut names, name, command, TaskSource::CargoAlias);
    }

    out
}

/// Find the command for `name`, using the same precedence as [`discover`].
pub fn find(root: &Path, name: &str) -> Option<Task> {
    discover(root).into_iter().find(|t| t.name == name)
}

fn add(
    out: &mut Vec<Task>,
    names: &mut Vec<String>,
    name: String,
    command: String,
    source: TaskSource,
) {
    if names.iter().any(|n| n == &name) {
        return;
    }
    names.push(name.clone());
    out.push(Task {
        name,
        command,
        source,
    });
}

fn package_json_scripts(root: &Path) -> Vec<(String, String)> {
    let raw = match std::fs::read_to_string(root.join("package.json")) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let value: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let Some(scripts) = value.get("scripts").and_then(|s| s.as_object()) else {
        return Vec::new();
    };
    scripts
        .iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
        .collect()
}

fn makefile_targets(root: &Path) -> Vec<String> {
    let raw = match std::fs::read_to_string(root.join("Makefile")) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for line in raw.lines() {
        // A target line looks like `name:` or `name: deps`, starting at column
        // 0 (recipe bodies are tab-indented). Skip special/pattern targets.
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        let Some((head, _)) = line.split_once(':') else {
            continue;
        };
        let name = head.trim();
        if name.is_empty()
            || name.starts_with('.')
            || name.contains('=')
            || name.contains('%')
            || name.contains('$')
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '/')
        {
            continue;
        }
        if !out.contains(&name.to_string()) {
            out.push(name.to_string());
        }
    }
    out
}

fn justfile_recipes(root: &Path) -> Vec<String> {
    let path = if root.join("justfile").is_file() {
        root.join("justfile")
    } else if root.join("Justfile").is_file() {
        root.join("Justfile")
    } else {
        return Vec::new();
    };
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for line in raw.lines() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((head, _)) = line.split_once(':') else {
            continue;
        };
        // Recipe head may carry parameters: `build target="x"` — take the name.
        let name = head.split_whitespace().next().unwrap_or("").trim();
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            continue;
        }
        if !out.contains(&name.to_string()) {
            out.push(name.to_string());
        }
    }
    out
}

fn cargo_aliases(root: &Path) -> Vec<String> {
    for rel in ["Cargo.toml", ".cargo/config.toml", ".cargo/config"] {
        let raw = match std::fs::read_to_string(root.join(rel)) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(value) = raw.parse::<toml::Value>() {
            if let Some(alias) = value.get("alias").and_then(|a| a.as_table()) {
                let names: Vec<String> = alias.keys().cloned().collect();
                if !names.is_empty() {
                    return names;
                }
            }
        }
    }
    Vec::new()
}

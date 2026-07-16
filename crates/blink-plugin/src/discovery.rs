use std::path::{Path, PathBuf};

use crate::error::{PluginError, Result};

/// The prefix every plugin executable must have, matching the convention
/// used by `cargo-*` and `git-*` subcommands.
const PLUGIN_PREFIX: &str = "blink-";

/// A discovered plugin: a name (without the `blink-` prefix) and the path
/// to its executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plugin {
    pub name: String,
    pub path: PathBuf,
}

/// Blink's managed plugin directory: `~/.blink/plugins`. This is where
/// `install_plugin` places plugins and where `find_plugin`/`discover_plugins`
/// look first, before falling back to the system `PATH`.
pub fn plugin_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or(PluginError::NoHomeDir)?;
    Ok(PathBuf::from(home).join(".blink").join("plugins"))
}

/// Find a plugin named `name` (without the `blink-` prefix), checking
/// Blink's managed plugin directory first, then every directory on `PATH`.
pub fn find_plugin(name: &str) -> Option<Plugin> {
    let exe_name = executable_name(name);

    if let Ok(dir) = plugin_dir() {
        let candidate = dir.join(&exe_name);
        if is_executable_file(&candidate) {
            return Some(Plugin {
                name: name.to_string(),
                path: candidate,
            });
        }
    }

    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var).find_map(|dir| {
        let candidate = dir.join(&exe_name);
        is_executable_file(&candidate).then_some(Plugin {
            name: name.to_string(),
            path: candidate,
        })
    })
}

/// List every plugin discoverable in Blink's managed plugin directory or
/// on `PATH`, deduplicated by name (managed-directory entries take
/// priority over same-named `PATH` entries, matching [`find_plugin`]'s
/// resolution order).
pub fn discover_plugins() -> Vec<Plugin> {
    let mut found = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut search_dirs = Vec::new();
    if let Ok(dir) = plugin_dir() {
        search_dirs.push(dir);
    }
    if let Some(path_var) = std::env::var_os("PATH") {
        search_dirs.extend(std::env::split_paths(&path_var));
    }

    for dir in search_dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !is_executable_file(&path) {
                continue;
            }
            let Some(name) = plugin_name_from_path(&path) else {
                continue;
            };
            if seen.insert(name.clone()) {
                found.push(Plugin { name, path });
            }
        }
    }

    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{PLUGIN_PREFIX}{name}.exe")
    } else {
        format!("{PLUGIN_PREFIX}{name}")
    }
}

fn plugin_name_from_path(path: &Path) -> Option<String> {
    let file_stem = path.file_stem()?.to_str()?;
    file_stem
        .strip_prefix(PLUGIN_PREFIX)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::plugin_name_from_path;

    #[test]
    fn extracts_plugin_name_from_executable_path() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(if cfg!(windows) {
            "blink-rust.exe"
        } else {
            "blink-rust"
        });

        assert_eq!(plugin_name_from_path(&path).as_deref(), Some("rust"));
    }

    #[test]
    fn ignores_files_without_the_blink_prefix() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cargo");

        assert_eq!(plugin_name_from_path(&path), None);
    }

    #[test]
    fn ignores_bare_blink_binary() {
        let dir = TempDir::new().unwrap();
        let path = dir
            .path()
            .join(if cfg!(windows) { "blink.exe" } else { "blink" });

        assert_eq!(plugin_name_from_path(&path), None);
    }
}

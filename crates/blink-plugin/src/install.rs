use std::path::{Path, PathBuf};

use crate::discovery::plugin_dir;
use crate::error::{PluginError, Result};

/// Install a plugin by copying an existing executable at `source` into
/// Blink's managed plugin directory as `blink-<name>`. There is no remote
/// plugin registry (nothing like `blink add rust-plugin` fetching from the
/// network) — this only ever copies a local file you already built or
/// downloaded, which is the only thing Blink can do honestly without a
/// registry to back it.
pub fn install_plugin(source: &Path, name: &str) -> Result<PathBuf> {
    install_plugin_to(source, name, &plugin_dir()?)
}

/// Like [`install_plugin`], but into an arbitrary directory instead of
/// Blink's managed plugin directory. Exists so tests can exercise the
/// install logic without touching the real `~/.blink/plugins`.
pub fn install_plugin_to(source: &Path, name: &str, dir: &Path) -> Result<PathBuf> {
    if !source.is_file() {
        return Err(PluginError::SourceNotFound(source.to_path_buf()));
    }

    std::fs::create_dir_all(dir).map_err(|source| PluginError::CreateDir {
        path: dir.to_path_buf(),
        source,
    })?;

    let exe_name = if cfg!(windows) {
        format!("blink-{name}.exe")
    } else {
        format!("blink-{name}")
    };
    let destination = dir.join(exe_name);

    std::fs::copy(source, &destination).map_err(|source| PluginError::Install {
        path: destination.clone(),
        source,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&destination) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(permissions.mode() | 0o111);
            let _ = std::fs::set_permissions(&destination, permissions);
        }
    }

    Ok(destination)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{install_plugin, install_plugin_to};

    #[test]
    fn errors_when_source_does_not_exist() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("does-not-exist");

        let result = install_plugin(&missing, "test");

        assert!(result.is_err());
    }

    #[test]
    fn copies_source_into_target_directory_with_prefixed_name() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let source = source_dir.path().join("my-tool");
        std::fs::write(&source, b"#!/bin/sh\necho hi\n").unwrap();

        let installed = install_plugin_to(&source, "hello", target_dir.path()).unwrap();

        let expected_name = if cfg!(windows) {
            "blink-hello.exe"
        } else {
            "blink-hello"
        };
        assert_eq!(installed, target_dir.path().join(expected_name));
        assert!(installed.is_file());
    }
}

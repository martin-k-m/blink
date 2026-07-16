use std::path::PathBuf;

use crate::error::{CacheError, Result};

/// The platform-appropriate per-user cache directory for Blink:
/// `~/.cache/blink` on Linux, `~/Library/Caches/blink` on macOS,
/// `%LOCALAPPDATA%/blink` on Windows. Created if it doesn't exist.
///
/// Unlike [`crate::Cache`]'s project-local `.blink/cache.json` (which
/// tracks file changes for `blink build`), this directory holds data that
/// makes sense to share *across* projects and across time: cached analysis
/// results keyed by project path, so re-running `blink analyze` on an
/// unchanged project doesn't repeat expensive work.
pub fn global_cache_dir() -> Result<PathBuf> {
    let base = platform_cache_base().ok_or(CacheError::NoCacheDir)?;
    let dir = base.join("blink");
    std::fs::create_dir_all(&dir).map_err(|source| CacheError::Write {
        path: dir.clone(),
        source,
    })?;
    Ok(dir)
}

fn platform_cache_base() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library").join("Caches"))
    } else if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("USERPROFILE")
                    .map(|h| PathBuf::from(h).join("AppData").join("Local"))
            })
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
    }
}

#[cfg(test)]
mod tests {
    use super::platform_cache_base;

    #[test]
    fn resolves_a_platform_cache_base() {
        // Every CI/dev environment sets at least one of HOME/USERPROFILE/
        // LOCALAPPDATA, so this should never be None in practice.
        assert!(platform_cache_base().is_some());
    }
}

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

use crate::error::{Result, ServerError};

const IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".cache",
    ".blink",
];

/// How long to wait after the last filesystem event in a burst before
/// notifying watchers, so a save-triggered flurry of writes collapses into
/// one rebuild.
const DEBOUNCE: Duration = Duration::from_millis(150);

/// Watches a project directory for file changes, debouncing bursts of
/// events and filtering out paths under ignored directories (build output,
/// dependency caches, VCS metadata, and Blink's own cache) so that saving
/// once triggers exactly one notification.
pub struct FileWatcher {
    // Held to keep the underlying OS watch alive for the lifetime of `self`.
    _debouncer: Debouncer<notify::RecommendedWatcher>,
    rx: Receiver<Vec<PathBuf>>,
    root: PathBuf,
}

impl FileWatcher {
    pub fn new(root: &Path) -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let watch_root = root.to_path_buf();

        let mut debouncer = new_debouncer(DEBOUNCE, move |result: DebounceEventResult| {
            let Ok(events) = result else { return };
            let paths: Vec<PathBuf> = events
                .into_iter()
                .map(|event| event.path)
                .filter(|path| !is_ignored(&watch_root, path))
                .collect();
            if !paths.is_empty() {
                let _ = tx.send(paths);
            }
        })
        .map_err(|source| ServerError::Watch {
            path: root.to_path_buf(),
            source,
        })?;

        debouncer
            .watcher()
            .watch(root, RecursiveMode::Recursive)
            .map_err(|source| ServerError::Watch {
                path: root.to_path_buf(),
                source,
            })?;

        Ok(Self {
            _debouncer: debouncer,
            rx,
            root: root.to_path_buf(),
        })
    }

    /// Block until the next debounced batch of changed files arrives.
    /// Returns `None` once the watcher has shut down.
    pub fn recv(&self) -> Option<Vec<PathBuf>> {
        self.rx.recv().ok()
    }

    /// Paths relative to the watched root, for display purposes.
    pub fn relativize(&self, paths: &[PathBuf]) -> Vec<String> {
        paths
            .iter()
            .map(|p| {
                p.strip_prefix(&self.root)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }
}

fn is_ignored(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .any(|c| IGNORED_DIRS.contains(&c.as_os_str().to_string_lossy().as_ref()))
}

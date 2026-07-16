//! Subprocess-based plugin discovery and execution for Blink, in the style
//! of `cargo`/`git` subcommands: a plugin is any executable named
//! `blink-<name>` on `PATH` (or in Blink's managed `~/.blink/plugins`
//! directory). `blink <name>` for an unrecognized built-in command execs
//! that plugin, forwarding the remaining arguments and inheriting stdio.
//!
//! There is no dynamic loading (no `libloading`, no in-process plugin
//! ABI) and no remote plugin registry. Both are real limitations, not
//! oversights: dynamic loading of native code is inherently
//! unsafe/ABI-fragile across Rust versions, and a registry would require
//! backing infrastructure Blink doesn't have. The subprocess convention is
//! simple, safe, and is exactly how mature tools (Cargo, Git, kubectl)
//! solve the same problem.

mod discovery;
mod error;
mod install;
mod run;

pub use discovery::{discover_plugins, find_plugin, plugin_dir, Plugin};
pub use error::{PluginError, Result};
pub use install::{install_plugin, install_plugin_to};
pub use run::run_plugin;

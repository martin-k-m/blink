use crate::discovery::Plugin;
use crate::error::{PluginError, Result};

/// Run a plugin, inheriting the current process's stdin/stdout/stderr, and
/// wait for it to finish. Returns the plugin's exit code (or `1` if it was
/// terminated by a signal rather than exiting normally, which has no
/// meaningful exit code on Unix).
pub fn run_plugin(plugin: &Plugin, args: &[String]) -> Result<i32> {
    let status = std::process::Command::new(&plugin.path)
        .args(args)
        .status()
        .map_err(|source| PluginError::Launch {
            name: plugin.name.clone(),
            source,
        })?;
    Ok(status.code().unwrap_or(1))
}

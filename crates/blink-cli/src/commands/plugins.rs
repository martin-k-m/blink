use std::path::PathBuf;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::{PluginsAction, PluginsArgs};
use crate::ui;

pub fn run(args: PluginsArgs) -> Result<()> {
    match args.action {
        None => list(),
        Some(PluginsAction::Install { source, name }) => install(source, name),
    }
}

fn list() -> Result<()> {
    let plugins = blink_plugin::discover_plugins();

    ui::banner("Blink Plugins");
    println!();

    if plugins.is_empty() {
        println!("  No plugins found.");
        println!();
        println!(
            "  {}",
            "Plugins are executables named `blink-<name>` on PATH, or in".dimmed()
        );
        println!("  {}", "~/.blink/plugins. Install one with:".dimmed());
        println!();
        println!(
            "  {}",
            "blink plugins install <path-to-executable> --name <name>".dimmed()
        );
    } else {
        for plugin in &plugins {
            ui::field(&plugin.name, plugin.path.display());
        }
    }

    println!();
    Ok(())
}

fn install(source: PathBuf, name: Option<String>) -> Result<()> {
    let name = name.unwrap_or_else(|| {
        source
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "plugin".to_string())
    });

    let installed = blink_plugin::install_plugin(&source, &name)
        .with_context(|| format!("could not install plugin from {}", source.display()))?;

    ui::banner("Blink Plugins");
    println!();
    ui::step(format!("Installed '{name}' to {}", installed.display()));
    println!();
    println!("  {} blink {name}", "Run it with:".dimmed());
    println!();
    Ok(())
}

use std::time::Instant;

use anyhow::{Context, Result};
use blink_cache::Cache;
use blink_core::BlinkConfig;
use colored::Colorize;

use crate::cli::BuildArgs;
use crate::ui;

pub fn run(args: BuildArgs) -> Result<()> {
    let start = Instant::now();

    ui::banner("Blink Build");
    println!("  Building...");
    println!();

    let cache_enabled = BlinkConfig::load(&args.path)
        .map(|c| c.optimization.cache)
        .unwrap_or(true);

    if !cache_enabled {
        let spinner = ui::spinner("Scanning files...");
        let current = Cache::scan(&args.path);
        spinner.finish_and_clear();
        ui::step(format!(
            "Scanned {} files (caching disabled in blink.toml)",
            current.file_count()
        ));
        ui::footer(
            "Build complete in:",
            format!("{}ms", start.elapsed().as_millis()),
        );
        return Ok(());
    }

    let previous = Cache::load(&args.path)
        .with_context(|| format!("could not read cache in {}", args.path.display()))?;
    let spinner = ui::spinner("Hashing files...");
    let current = Cache::scan(&args.path);
    spinner.finish_and_clear();

    match &previous {
        Some(prev) => {
            let diff = current.diff(prev);
            ui::step(format!("Compared {} files against cache", diff.total));
            println!();
            println!("  {}", "Cache".bold());
            println!("    {} unchanged", diff.unchanged);
            println!("    {} changed", diff.changed);
            println!("    {} added", diff.added);
            println!("    {} removed", diff.removed);
        }
        None => {
            ui::step(format!(
                "Scanned {} files (no previous cache)",
                current.file_count()
            ));
        }
    }

    current
        .save(&args.path)
        .with_context(|| format!("could not write cache in {}", args.path.display()))?;
    ui::step("Cache saved");

    ui::footer(
        "Build complete in:",
        format!("{}ms", start.elapsed().as_millis()),
    );
    Ok(())
}

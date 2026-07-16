use anyhow::{Context, Result};
use blink_core::{BlinkConfig, ProjectDetector};

use crate::cli::InitArgs;
use crate::ui;

pub fn run(args: InitArgs) -> Result<()> {
    std::fs::create_dir_all(&args.path)
        .with_context(|| format!("could not create {}", args.path.display()))?;

    ui::banner("Blink");
    println!("  Creating project...");

    let name = ProjectDetector::new()
        .detect(&args.path)
        .map(|p| p.name)
        .unwrap_or_else(|_| fallback_name(&args.path));
    ui::step("Detected environment");

    let config = BlinkConfig::new(&name);
    config
        .write(&args.path)
        .with_context(|| format!("could not write blink.toml in {}", args.path.display()))?;
    ui::step("Created configuration");
    ui::step("Ready");

    println!();
    println!("  Project initialized.");
    Ok(())
}

fn fallback_name(path: &std::path::Path) -> String {
    let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    absolute
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "blink-project".to_string())
}

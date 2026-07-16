use std::time::Instant;

use anyhow::{Context, Result};
use blink_core::{effective_ignored_dirs, ProjectDetector};
use colored::Colorize;

use crate::cli::ScanArgs;
use crate::ui;

pub fn run(args: ScanArgs) -> Result<()> {
    let spinner = ui::spinner("Scanning project...");

    if args.verbose {
        spinner.finish_and_clear();
        ui::banner("Blink Scanner");
        println!();
        ui::field("Scanning", args.path.display());
    }

    let start = Instant::now();
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not scan {}", args.path.display()))?;
    let elapsed = start.elapsed();
    spinner.finish_and_clear();

    if args.verbose {
        ui::field("Files discovered", ui::format_count(project.file_count));
        ui::field("Detected", format!("{} Project", project.language));
        ui::field("Configuration", &project.config_file);
        if project.is_workspace {
            ui::field("Workspace", "yes");
        }
        if project.has_virtualenv {
            ui::field("Virtualenv", "detected");
        }
        println!();
        println!("  {}", "Ignore rules in effect:".dimmed());
        for dir in effective_ignored_dirs(&args.path) {
            println!("    {} {dir}", "-".dimmed());
        }
        ui::footer("Completed:", format!("{}ms", elapsed.as_millis()));
        return Ok(());
    }

    ui::banner("Blink Project Scanner");
    ui::field("Project", &project.name);
    ui::field("Framework", project.framework);
    ui::field("Language", project.language);
    ui::field("Package manager", project.package_manager);
    ui::field("Files", ui::format_count(project.file_count));
    ui::field("Dependencies", ui::format_count(project.dependency_count()));

    ui::footer("Scan completed:", format!("{}ms", elapsed.as_millis()));
    Ok(())
}

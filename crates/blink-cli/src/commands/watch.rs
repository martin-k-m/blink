use std::time::Instant;

use anyhow::{Context, Result};
use blink_analyzer::Analyzer;
use blink_core::ProjectDetector;
use blink_server::FileWatcher;
use colored::Colorize;

use crate::cli::WatchArgs;
use crate::ui;

pub fn run(args: WatchArgs) -> Result<()> {
    ui::banner("Blink Watch");
    println!();

    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;
    println!("  {} {}", "Monitoring:".dimmed(), args.path.display());
    println!("  {} {}", "Project:".dimmed(), project.name);
    println!();
    println!(
        "  {}",
        "Watching for file changes... (Ctrl+C to stop)".dimmed()
    );

    let watcher = FileWatcher::new(&args.path).context("could not start file watcher")?;

    while let Some(paths) = watcher.recv() {
        let relative = watcher.relativize(&paths);
        for path in &relative {
            println!();
            println!("Change detected:");
            println!();
            println!("  {path}");
        }

        let start = Instant::now();
        println!();
        println!("  Updating analysis...");
        if let Ok(project) = ProjectDetector::new().detect(&args.path) {
            let report = Analyzer::new().analyze(&project, &args.path);
            let issues = blink_report::issues(&report);
            if issues.is_empty() {
                ui::step("No issues detected");
            } else {
                for issue in &issues {
                    ui::warning(&issue.summary);
                }
            }
        }
        ui::footer("Complete:", format!("{}ms", start.elapsed().as_millis()));
    }

    Ok(())
}

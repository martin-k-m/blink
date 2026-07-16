use anyhow::{Context, Result};
use blink_analyzer::compute_health;
use blink_core::ProjectDetector;
use blink_report::health_bar;
use colored::Colorize;

use crate::analysis::analyze_cached;
use crate::cli::HealthArgs;
use crate::ui;

pub fn run(args: HealthArgs) -> Result<()> {
    let spinner = (!args.json).then(|| ui::spinner("Checking project health..."));

    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;
    let analysis = analyze_cached(&project, &args.path, false).report;
    let health = compute_health(&analysis, &args.path);

    if let Some(spinner) = &spinner {
        spinner.finish_and_clear();
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&health)?);
        return Ok(());
    }

    ui::banner("Blink Project Health");
    println!();
    println!("  {}", "Score".bold());
    println!("    {}", health_bar(health.overall, 10));

    println!();
    ui::field(
        "Code Organization",
        format!("{}/100", health.code_organization),
    );
    ui::field("Dependencies", format!("{}/100", health.dependencies));
    ui::field("Configuration", format!("{}/100", health.configuration));

    if !health.suggestions.is_empty() {
        println!();
        println!("  {}", "Suggestions".bold());
        for suggestion in &health.suggestions {
            ui::suggestion(suggestion);
        }
    }

    println!();
    Ok(())
}

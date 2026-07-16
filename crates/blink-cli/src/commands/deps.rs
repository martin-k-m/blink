use anyhow::{Context, Result};
use blink_core::ProjectDetector;
use blink_report::{dependency_stats_table, largest_packages_table};
use colored::Colorize;

use crate::analysis::analyze_cached;
use crate::cli::DepsArgs;
use crate::ui;

pub fn run(args: DepsArgs) -> Result<()> {
    let spinner = ui::spinner("Analyzing dependencies...");

    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;
    let report = analyze_cached(&project, &args.path, false).report;
    spinner.finish_and_clear();

    ui::banner("Blink Dependency Analysis");
    println!();
    print_indented(&dependency_stats_table(&report), "  ");

    if let Some(table) = largest_packages_table(&report) {
        println!();
        println!("  {}", "Largest Dependencies".bold());
        print_indented(&table, "  ");
    }

    let issues = blink_report::issues(&report);
    println!();
    if issues.is_empty() {
        println!("  {} No issues detected", "\u{2713}".green().bold());
    } else {
        println!("  {}", "Issues".bold());
        for issue in &issues {
            ui::warning(&issue.summary);
        }
    }

    println!();
    Ok(())
}

fn print_indented(block: &str, indent: &str) {
    for line in block.lines() {
        println!("{indent}{line}");
    }
}

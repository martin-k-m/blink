use anyhow::{Context, Result};
use blink_core::ProjectDetector;
use blink_report::{
    dependency_stats_table, health_bar, largest_packages_table, project_type_label,
};
use colored::Colorize;

use crate::analysis::analyze_cached;
use crate::cli::AnalyzeArgs;
use crate::ui;

pub fn run(args: AnalyzeArgs) -> Result<()> {
    let spinner = (!args.json).then(|| ui::spinner("Analyzing project..."));

    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;
    let outcome = analyze_cached(&project, &args.path, args.online);
    let report = outcome.report;

    if let Some(spinner) = &spinner {
        spinner.finish_and_clear();
    }

    if args.json {
        let json_report = blink_report::to_json_report(&project, &report);
        println!("{}", serde_json::to_string_pretty(&json_report)?);
        return Ok(());
    }

    ui::banner("Blink Analysis");

    println!();
    ui::field("Project", &project.name);
    ui::field("Type", project_type_label(&project));
    ui::field("Files", ui::format_count(project.file_count));

    println!();
    println!("  {}", "Health".bold());
    println!("    {}", health_bar(report.health_score(), 10));

    println!();
    println!("  {}", "Dependencies".bold());
    print_indented(&dependency_stats_table(&report), "  ");

    if let Some(table) = largest_packages_table(&report) {
        println!();
        println!("  {}", "Largest Packages".bold());
        print_indented(&table, "  ");
    }

    let issues = blink_report::issues(&report);
    if issues.is_empty() {
        println!();
        println!("  {} No issues detected", "\u{2713}".green().bold());
    } else {
        println!();
        println!("  {}", "Potential Issues".bold());
        for issue in &issues {
            ui::warning(&issue.summary);
        }
        if !report.outdated_checked {
            println!(
                "    {} Outdated packages: unknown ({})",
                "-".dimmed(),
                "run with --online to check".dimmed()
            );
        }
    }

    let recommendations = report.recommendations();
    if !recommendations.is_empty() {
        println!();
        println!("  {}", "Suggestions".bold());
        for rec in &recommendations {
            ui::suggestion(rec);
        }
    }

    println!();
    println!("  {}", "Performance".bold());
    if outcome.from_cache {
        println!("    {}", "Using cache".dimmed());
    }
    println!("    Analysis time    {}ms", outcome.elapsed_ms);
    match report.build_output_bytes {
        Some(bytes) => println!(
            "    Build output     {}",
            blink_analyzer::format_bytes(bytes)
        ),
        None => println!("    Build output     {}", "not built yet".dimmed()),
    }

    println!();
    Ok(())
}

/// Print a (possibly multi-line) block with every line prefixed by `indent`.
fn print_indented(block: &str, indent: &str) {
    for line in block.lines() {
        println!("{indent}{line}");
    }
}

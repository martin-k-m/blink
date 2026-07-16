use anyhow::{Context, Result};
use blink_analyzer::{format_bytes, Analyzer};
use blink_core::ProjectDetector;
use colored::Colorize;

use crate::cli::AnalyzeArgs;
use crate::ui;

pub fn run(args: AnalyzeArgs) -> Result<()> {
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("no recognizable project in {}", args.path.display()))?;
    let report = Analyzer::new()
        .online(args.online)
        .analyze(&project, &args.path);

    ui::banner("Blink Analysis Report");

    println!();
    println!("  {}", "Dependencies".bold());
    println!(
        "    {} Healthy packages: {}",
        "\u{2713}".green().bold(),
        report.healthy_count()
    );
    if !report.unused.is_empty() {
        println!(
            "    {} Unused packages: {}",
            "\u{26a0}".yellow().bold(),
            report.unused.len()
        );
    }
    if !report.duplicates.is_empty() {
        println!(
            "    {} Duplicate versions: {}",
            "\u{26a0}".yellow().bold(),
            report.duplicates.len()
        );
    }
    if report.outdated_checked {
        println!(
            "    {} Outdated packages: {}",
            "\u{26a0}".yellow().bold(),
            report.outdated.len()
        );
    } else {
        println!(
            "    {} Outdated packages: unknown ({})",
            "-".dimmed(),
            "run with --online to check".dimmed()
        );
    }

    println!();
    println!("  {}", "Performance".bold());
    println!("    Analysis time    {}ms", report.elapsed_ms);
    match report.build_output_bytes {
        Some(bytes) => println!("    Build output     {}", format_bytes(bytes)),
        None => println!("    Build output     {}", "not built yet".dimmed()),
    }

    let recommendations = report.recommendations();
    if !recommendations.is_empty() {
        println!();
        println!("  {}", "Recommendations".bold());
        for rec in &recommendations {
            println!("    {} {}", "\u{2192}".truecolor(255, 138, 0), rec);
        }
    }

    println!();
    Ok(())
}

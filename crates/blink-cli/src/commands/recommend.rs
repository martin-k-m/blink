use anyhow::{Context, Result};
use blink_analyzer::{RecommendationCategory, RecommendationEngine, Status};
use blink_core::ProjectDetector;
use colored::Colorize;

use crate::analysis::analyze_cached;
use crate::cli::RecommendArgs;
use crate::ui;

pub fn run(args: RecommendArgs) -> Result<()> {
    let spinner = ui::spinner("Evaluating recommendations...");

    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;
    let analysis = analyze_cached(&project, &args.path, args.online).report;
    let vulnerabilities = args
        .online
        .then(|| blink_analyzer::find_vulnerabilities(&project));
    let recommendations =
        RecommendationEngine::evaluate(&analysis, &args.path, vulnerabilities.as_deref());
    spinner.finish_and_clear();

    ui::banner("Blink Recommendations");

    for category in [
        RecommendationCategory::Performance,
        RecommendationCategory::Maintenance,
        RecommendationCategory::Security,
    ] {
        let entries: Vec<_> = recommendations
            .iter()
            .filter(|r| r.category == category)
            .collect();
        if entries.is_empty() {
            continue;
        }

        println!();
        println!("  {}", category_label(category).bold());
        for entry in entries {
            let icon = match entry.status {
                Status::Ok => "\u{2713}".green().bold(),
                Status::Warning => "\u{26a0}".yellow().bold(),
                Status::Unknown => "-".dimmed(),
            };
            println!("    {icon} {}", entry.message);
        }
    }

    println!();
    Ok(())
}

fn category_label(category: RecommendationCategory) -> &'static str {
    match category {
        RecommendationCategory::Performance => "Performance",
        RecommendationCategory::Maintenance => "Maintenance",
        RecommendationCategory::Security => "Security",
    }
}

use anyhow::{Context, Result};
use blink_core::ProjectDetector;
use blink_workflow::optimize::{self, OptStatus};
use colored::Colorize;

use crate::analysis::analyze_cached;
use crate::cli::FormatArgs;
use crate::indexing::ensure_index;
use crate::ui;

pub fn run(args: FormatArgs) -> Result<()> {
    let spinner = ui::spinner("Analyzing project...");
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;
    let report = analyze_cached(&project, &args.path, false).report;
    let index = ensure_index(&args.path)?;
    spinner.finish_and_clear();

    let opt = optimize::optimize(&project, &report, &index, &args.path);

    if args.json {
        let json = serde_json::json!({
            "score": opt.score,
            "warnings": opt.warnings(),
            "checks": opt.checks.iter().map(|c| serde_json::json!({
                "category": c.category,
                "status": match c.status { OptStatus::Good => "good", OptStatus::Warn => "warn" },
                "detail": c.detail,
            })).collect::<Vec<_>>(),
            "suggestions": opt.suggestions,
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Optimization Report");
    println!();
    println!(
        "  {} {}",
        "Project score".dimmed(),
        format!("{}/100", opt.score).bold()
    );

    println!();
    for check in &opt.checks {
        match check.status {
            OptStatus::Good => {
                println!(
                    "  {} {:<18}  {}",
                    "\u{2713}".green().bold(),
                    check.category,
                    check.detail.dimmed()
                );
            }
            OptStatus::Warn => {
                println!(
                    "  {} {:<18}  {}",
                    "\u{26a0}".yellow().bold(),
                    check.category,
                    check.detail
                );
            }
        }
    }

    if !opt.suggestions.is_empty() {
        println!();
        println!("  {}", "Suggestions".bold());
        for suggestion in &opt.suggestions {
            ui::suggestion(suggestion);
        }
    }

    println!();
    Ok(())
}

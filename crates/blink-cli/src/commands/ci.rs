use anyhow::Result;
use blink_analyzer::{
    compute_health, find_vulnerabilities, RecommendationCategory, RecommendationEngine, Status,
};
use blink_core::ProjectDetector;
use colored::Colorize;

use crate::analysis::analyze_cached;
use crate::cli::CiArgs;
use crate::ui;

/// Exit codes for `blink ci`, distinct from every other command's plain
/// 0-success/1-error convention: a CI pipeline needs to distinguish
/// "clean," "clean but worth a look," and "something's actually broken."
const EXIT_PASS: i32 = 0;
const EXIT_WARNINGS: i32 = 1;
const EXIT_FAILURE: i32 = 2;

pub fn run(args: CiArgs) -> Result<()> {
    ui::banner("Blink CI Report");
    println!();

    let project = match ProjectDetector::new().detect(&args.path) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("  {} {err}", "error:".red().bold());
            std::process::exit(EXIT_FAILURE);
        }
    };

    let analysis = analyze_cached(&project, &args.path, args.online).report;
    let health = compute_health(&analysis, &args.path);
    // `None` covers both "not asked for" and "the audit couldn't run"; either
    // way the security verdict is unknown, never a silent pass.
    let vulnerabilities = args
        .online
        .then(|| find_vulnerabilities(&project))
        .flatten();
    let recommendations =
        RecommendationEngine::evaluate(&analysis, &args.path, vulnerabilities.as_deref());

    let critical = vulnerabilities.as_ref().map_or(0, Vec::len);
    let warnings = recommendations
        .iter()
        .filter(|r| r.status == Status::Warning && r.category != RecommendationCategory::Security)
        .count();

    ui::field("Health", format!("{}/100", health.overall));
    println!();
    println!("  {}", "Issues".bold());
    println!("    {critical} critical");
    println!("    {warnings} warnings");
    println!();

    let code = if critical > 0 {
        println!("  {}", "Build failed".red().bold());
        EXIT_FAILURE
    } else if warnings > 0 {
        println!("  {}", "Build passed with warnings".yellow().bold());
        EXIT_WARNINGS
    } else {
        println!("  {}", "Build passed".green().bold());
        EXIT_PASS
    };
    println!();

    std::process::exit(code);
}

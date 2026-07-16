use anyhow::{Context, Result};
use blink_analyzer::find_vulnerabilities;
use blink_core::ProjectDetector;
use colored::Colorize;

use crate::cli::SecurityArgs;
use crate::ui;

pub fn run(args: SecurityArgs) -> Result<()> {
    let spinner =
        (!args.json).then(|| ui::spinner("Checking OSV.dev for known vulnerabilities..."));

    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;
    let vulnerabilities = find_vulnerabilities(&project);

    if let Some(spinner) = &spinner {
        spinner.finish_and_clear();
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&vulnerabilities)?);
        return Ok(());
    }

    ui::banner("Blink Security");
    println!();

    if vulnerabilities.is_empty() {
        println!(
            "  {} No known vulnerabilities found in {} declared {}",
            "\u{2713}".green().bold(),
            project.dependency_count(),
            if project.dependency_count() == 1 {
                "dependency"
            } else {
                "dependencies"
            }
        );
    } else {
        println!("  {}", "Vulnerable dependencies".bold());
        for pkg in &vulnerabilities {
            println!(
                "    {} {} {} — {}",
                "\u{26a0}".yellow().bold(),
                pkg.name,
                pkg.version.dimmed(),
                pkg.ids.join(", ")
            );
        }
    }

    println!();
    println!(
        "  {}",
        "Source: osv.dev (Open Source Vulnerabilities). IDs are not fetched in detail — look them up at osv.dev/vulnerability/<id>.".dimmed()
    );
    println!();
    Ok(())
}

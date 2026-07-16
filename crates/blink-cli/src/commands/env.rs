use anyhow::Result;
use blink_workflow::env;
use colored::Colorize;

use crate::cli::FormatArgs;
use crate::ui;

pub fn run(args: FormatArgs) -> Result<()> {
    let report = env::compare(&args.path);

    if args.json {
        let json = serde_json::json!({
            "has_example": report.has_example,
            "has_env": report.has_env,
            "configured": report.configured,
            "missing": report.missing,
            "unused": report.unused,
            "complete": report.is_complete(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Environment");
    if !report.has_example && !report.has_env {
        println!("\n  No .env or .env.example found.\n");
        return Ok(());
    }
    println!();

    for name in &report.configured {
        println!("  {} {}", "\u{2713}".green().bold(), name);
    }
    for name in &report.missing {
        println!(
            "  {} {}  {}",
            "\u{26a0}".yellow().bold(),
            name.bold(),
            "declared in .env.example, missing from .env".dimmed()
        );
    }
    for name in &report.unused {
        println!(
            "  {} {}  {}",
            "\u{2192}".dimmed(),
            name,
            "in .env but not in .env.example".dimmed()
        );
    }

    println!();
    if report.is_complete() {
        ui::footer("Result", "all required variables are set");
    } else {
        ui::footer(
            "Result",
            format!("{} variable(s) missing from .env", report.missing.len()),
        );
    }
    Ok(())
}

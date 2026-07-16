use anyhow::{Context, Result};
use blink_core::ProjectDetector;
use blink_workflow::doctor::{self, CheckStatus};
use colored::Colorize;

use crate::cli::FormatArgs;
use crate::ui;

pub fn run(args: FormatArgs) -> Result<()> {
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not inspect {}", args.path.display()))?;
    let report = doctor::diagnose(&project, &args.path);

    if args.json {
        let json = serde_json::json!({
            "healthy": report.is_healthy(),
            "missing_required": report.missing_required(),
            "missing_optional": report.missing_optional(),
            "checks": report.checks.iter().map(|c| serde_json::json!({
                "name": c.name,
                "status": match c.status { CheckStatus::Ok => "ok", CheckStatus::Missing => "missing" },
                "required": c.required,
                "detail": c.detail,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Doctor");
    println!();
    for check in &report.checks {
        match check.status {
            CheckStatus::Ok => {
                println!(
                    "  {} {}  {}",
                    "\u{2713}".green().bold(),
                    check.name,
                    check.detail.dimmed()
                );
            }
            CheckStatus::Missing if check.required => {
                println!(
                    "  {} {}  {}",
                    "\u{274c}".red().bold(),
                    check.name.bold(),
                    check.detail.dimmed()
                );
            }
            CheckStatus::Missing => {
                println!(
                    "  {} {}  {}",
                    "\u{26a0}".yellow().bold(),
                    check.name,
                    check.detail.dimmed()
                );
            }
        }
    }

    println!();
    if report.is_healthy() {
        ui::footer("Result", "environment ready");
    } else {
        ui::footer(
            "Result",
            format!("{} required tool(s) missing", report.missing_required()),
        );
    }
    Ok(())
}

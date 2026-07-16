use anyhow::Result;
use blink_analyzer::format_bytes;
use blink_workflow::filesystem;
use colored::Colorize;

use crate::cli::FormatArgs;
use crate::ui;

pub fn run(args: FormatArgs) -> Result<()> {
    let spinner = ui::spinner("Measuring storage...");
    let report = filesystem::analyze(&args.path);
    spinner.finish_and_clear();

    if args.json {
        let json = serde_json::json!({
            "total_bytes": report.total_bytes,
            "ignored_bytes": report.ignored_bytes,
            "source_bytes": report.source_bytes(),
            "entries": report.entries.iter().map(|e| serde_json::json!({
                "name": e.name,
                "bytes": e.bytes,
                "ignored": e.ignored,
                "is_dir": e.is_dir,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Filesystem");
    ui::field("Total size", format_bytes(report.total_bytes));
    ui::field("Source (kept)", format_bytes(report.source_bytes()));
    ui::field(
        "Regenerable",
        format!(
            "{} ({} reclaimable via `blink clean`)",
            format_bytes(report.ignored_bytes),
            format_bytes(report.ignored_bytes)
        ),
    );

    println!();
    println!("  {}", "Top-level usage".bold());
    for entry in report.entries.iter().take(15) {
        let marker = if entry.ignored {
            " (ignored)".dimmed().to_string()
        } else {
            String::new()
        };
        println!(
            "    {:>10}  {}{}",
            format_bytes(entry.bytes),
            entry.name,
            marker
        );
    }
    println!();
    Ok(())
}

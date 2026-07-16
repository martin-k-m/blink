use anyhow::Result;
use blink_analyzer::format_bytes;
use blink_workflow::duplicates;
use colored::Colorize;

use crate::cli::FormatArgs;
use crate::indexing::ensure_index;
use crate::ui;

pub fn run(args: FormatArgs) -> Result<()> {
    let index = ensure_index(&args.path)?;
    let groups = duplicates::find(&index);
    let wasted = duplicates::total_wasted(&groups);

    if args.json {
        let json = serde_json::json!({
            "groups": groups.iter().map(|g| serde_json::json!({
                "bytes": g.bytes,
                "wasted_bytes": g.wasted_bytes(),
                "paths": g.paths,
            })).collect::<Vec<_>>(),
            "total_wasted_bytes": wasted,
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Duplicate Files");
    if groups.is_empty() {
        println!(
            "\n  {} No duplicate files found\n",
            "\u{2713}".green().bold()
        );
        return Ok(());
    }

    println!();
    for group in &groups {
        println!(
            "  {} identical copies, {} each ({} reclaimable)",
            group.paths.len(),
            format_bytes(group.bytes),
            format_bytes(group.wasted_bytes()).bold()
        );
        for path in &group.paths {
            println!("    {} {path}", "•".dimmed());
        }
        println!();
    }

    ui::footer("Reclaimable", format_bytes(wasted));
    Ok(())
}

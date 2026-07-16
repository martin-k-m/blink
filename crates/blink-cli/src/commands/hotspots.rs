use anyhow::Result;
use blink_analyzer::format_bytes;
use blink_workflow::git;
use colored::Colorize;

use crate::cli::{HotspotsArgs, TimelineArgs};
use crate::indexing::ensure_index;
use crate::ui;

pub fn hotspots(args: HotspotsArgs) -> Result<()> {
    let index = ensure_index(&args.path)?;
    let largest = index.largest_files(args.limit);
    let churn = git::churn(&args.path);
    let churn_top: Vec<&(String, usize)> = churn.iter().take(args.limit).collect();

    if args.json {
        let json = serde_json::json!({
            "largest": largest.iter().map(|f| serde_json::json!({
                "path": f.path, "bytes": f.size, "lines": f.lines,
            })).collect::<Vec<_>>(),
            "most_changed": churn_top.iter().map(|(path, n)| serde_json::json!({
                "path": path, "changes": n,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Hotspots");
    println!();
    println!("  {}", "Largest files".bold());
    for file in &largest {
        println!("    {}  {}", format_bytes(file.size).dimmed(), file.path);
    }

    println!();
    println!("  {}", "Most frequently changed".bold());
    if churn_top.is_empty() {
        ui::field("", "no Git history available");
    } else {
        for (path, changes) in churn_top {
            println!("    {}  {}", format!("{changes}×").dimmed(), path);
        }
    }
    println!();
    Ok(())
}

pub fn timeline(args: TimelineArgs) -> Result<()> {
    ui::banner("Blink Timeline");

    if !git::available(&args.path) {
        println!();
        ui::field("Git", "not a git repository (or git unavailable)");
        println!();
        return Ok(());
    }

    if let Some(count) = git::commit_count(&args.path) {
        ui::field("Commits", ui::format_count(count));
    }

    let recent = git::recent_files(&args.path, args.limit);
    println!();
    println!("  {}", "Recently changed files".bold());
    if recent.is_empty() {
        ui::field("", "no changes found");
    } else {
        for path in &recent {
            println!("    {path}");
        }
    }

    let commits = git::recent_commits(&args.path, args.limit.min(10));
    if !commits.is_empty() {
        println!();
        println!("  {}", "Recent commits".bold());
        for subject in &commits {
            println!("    {} {subject}", "•".dimmed());
        }
    }
    println!();
    Ok(())
}

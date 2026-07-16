use anyhow::Result;
use blink_analyzer::format_bytes;
use blink_workflow::clean;
use colored::Colorize;

use crate::cli::CleanArgs;
use crate::proc;
use crate::ui;

pub fn run(args: CleanArgs) -> Result<()> {
    let all_targets = clean::plan(&args.path);

    // Heavy artifacts (target/, node_modules/, virtualenvs) only clean with
    // --all, since they cost a recompile/reinstall to restore.
    let targets: Vec<_> = all_targets
        .into_iter()
        .filter(|t| args.all || !t.heavy)
        .collect();
    let total = clean::total_bytes(&targets);

    ui::banner("Blink Clean");
    if targets.is_empty() {
        println!("\n  {} Nothing to clean\n", "\u{2713}".green().bold());
        if !args.all {
            ui::suggestion("Use `--all` to also remove target/, node_modules/, and virtualenvs.");
            println!();
        }
        return Ok(());
    }

    println!();
    for target in &targets {
        let heavy = if target.heavy {
            " (heavy)".yellow().to_string()
        } else {
            String::new()
        };
        println!(
            "    {:>10}  {}{}",
            format_bytes(target.bytes),
            display_path(&args.path, target),
            heavy
        );
    }
    ui::footer("Total", format_bytes(total));

    if args.dry_run {
        println!();
        ui::step("dry run \u{2014} nothing was deleted");
        println!();
        return Ok(());
    }

    println!();
    if !args.yes && !proc::confirm(&format!("Delete these {} item(s)?", targets.len())) {
        ui::step("cancelled");
        println!();
        return Ok(());
    }

    let mut removed = 0u64;
    for target in &targets {
        match std::fs::remove_dir_all(&target.path) {
            Ok(()) => {
                removed += target.bytes;
                ui::step(format!("removed {}", display_path(&args.path, target)));
            }
            Err(err) => {
                ui::warning(format!(
                    "could not remove {}: {err}",
                    display_path(&args.path, target)
                ));
            }
        }
    }

    ui::footer("Reclaimed", format_bytes(removed));
    Ok(())
}

/// Show the target path relative to the project root when possible, so output
/// stays readable in nested (`__pycache__`) cases.
fn display_path(root: &std::path::Path, target: &clean::CleanTarget) -> String {
    target
        .path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| target.name.clone())
}

use std::path::Path;

use anyhow::{bail, Context as _, Result};
use blink_context::ContextGraph;
use blink_core::{BlinkConfig, ProjectDetector};
use colored::Colorize;

use crate::cli::FormatArgs;
use crate::indexing::ensure_index;
use crate::ui;

/// Build the project's context graph, reusing the warm index. Shared by every
/// context-engine command (`context`/`query`/`explain`/`map`/`export`).
pub fn build(path: &Path) -> Result<ContextGraph> {
    // Honor `[context].enabled = false`.
    if let Ok(config) = BlinkConfig::load(path) {
        if !config.context.enabled {
            bail!(
                "the context engine is disabled for this project \
                 (`[context].enabled = false` in {}). Remove or set it to true to use `blink context`.",
                config_name(path)
            );
        }
    }

    let project = ProjectDetector::new()
        .detect(path)
        .with_context(|| format!("could not read a project at {}", path.display()))?;
    let index = ensure_index(path)?;
    let config = BlinkConfig::load(path).ok();
    Ok(ContextGraph::from_parts(
        path,
        &project,
        &index,
        config.as_ref(),
    ))
}

fn config_name(path: &Path) -> String {
    BlinkConfig::config_path(path)
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "blink.toml".to_string())
}

pub fn run(args: FormatArgs) -> Result<()> {
    let spinner = ui::spinner("Building context...");
    let graph = build(&args.path)?;
    spinner.finish_and_clear();

    if args.json {
        println!("{}", serde_json::to_string_pretty(&graph)?);
        return Ok(());
    }

    let p = &graph.project;
    ui::banner(&format!("Blink Context \u{2014} {}", p.name));

    // Identity.
    let mut descriptor = p.language.clone();
    if let Some(fw) = &p.framework {
        descriptor = format!("{descriptor} / {fw}");
    }
    if p.is_workspace {
        descriptor.push_str(" workspace");
    }
    ui::field("Project", descriptor);
    ui::field("Package manager", &p.package_manager);
    match &graph.config.file {
        Some(f) => ui::field("Config", f),
        None => ui::field("Config", "none (defaults)"),
    }

    // Measured statistics.
    println!();
    ui::field(
        "Files",
        format!(
            "{} ({} source)",
            ui::format_count(graph.stats.files),
            ui::format_count(graph.stats.source_files)
        ),
    );
    ui::field("Lines of code", ui::format_count(graph.stats.lines));
    ui::field("Symbols", ui::format_count(graph.stats.symbols));
    ui::field("Size", human_size(graph.stats.size_bytes));
    ui::field("Dependencies", ui::format_count(graph.dependencies.len()));
    ui::field("References", ui::format_count(graph.references.len()));

    // Important areas (ranked by symbol density).
    let areas = graph.ranked_areas();
    if !areas.is_empty() {
        println!();
        println!("  {}", "Important areas".bold());
        for area in areas.iter().take(8) {
            println!(
                "    {}  {}",
                area.path.bold(),
                format!("({} files · {} symbols)", area.files, area.symbols).dimmed()
            );
        }
    }

    // Commands.
    if !graph.commands.is_empty() {
        println!();
        println!("  {}", "Commands".bold());
        for cmd in graph.commands.iter().take(8) {
            println!(
                "    {}  {}",
                cmd.name.bold(),
                format!("({})", cmd.source).dimmed()
            );
        }
    }

    println!();
    ui::footer(
        "Next",
        "blink map  ·  blink query <text>  ·  blink explain <file>",
    );
    Ok(())
}

/// Human-readable byte size, shared by the context-engine commands.
pub fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

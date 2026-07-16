use anyhow::{bail, Result};
use blink_export::ExportFormat;
use colored::Colorize;

use crate::cli::MapArgs;
use crate::commands::context;
use crate::ui;

pub fn run(args: MapArgs) -> Result<()> {
    let spinner = ui::spinner("Mapping architecture...");
    let graph = context::build(&args.path)?;
    spinner.finish_and_clear();

    match args.format.trim().to_ascii_lowercase().as_str() {
        "terminal" | "text" => render_terminal(&graph),
        "graph" | "mermaid" | "mmd" => {
            print!("{}", blink_export::export(&graph, ExportFormat::Graph));
        }
        "json" => {
            let json = serde_json::json!({
                "project": graph.project.name,
                "areas": graph.ranked_areas().iter().map(|a| serde_json::json!({
                    "path": a.path, "files": a.files, "lines": a.lines,
                    "symbols": a.symbols, "languages": a.languages,
                })).collect::<Vec<_>>(),
                "edges": graph.area_edges().iter().map(|e| serde_json::json!({
                    "from": e.from, "to": e.to, "count": e.count,
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        "markdown" | "md" => render_markdown(&graph),
        other => bail!("unknown --format '{other}' (expected: terminal, markdown, json, graph)"),
    }
    Ok(())
}

fn render_terminal(graph: &blink_context::ContextGraph) {
    ui::banner(&format!(
        "Blink Architecture \u{2014} {}",
        graph.project.name
    ));

    let areas = graph.ranked_areas();
    println!();
    println!("  {}", "Areas".bold());
    if areas.is_empty() {
        println!("    {}", "no areas detected".dimmed());
    } else {
        let width = areas
            .iter()
            .take(15)
            .map(|a| a.path.len())
            .max()
            .unwrap_or(0);
        for area in areas.iter().take(15) {
            println!(
                "    {:<width$}  {}",
                area.path.bold(),
                format!("{} files · {} symbols", area.files, area.symbols).dimmed(),
                width = width
            );
        }
    }

    let edges = graph.area_edges();
    println!();
    println!("  {}", "Area dependencies".bold());
    if edges.is_empty() {
        println!(
            "    {}",
            "no cross-area references resolved (imports stay within areas, or aren't resolvable)"
                .dimmed()
        );
    } else {
        let width = edges
            .iter()
            .take(20)
            .map(|e| e.from.len())
            .max()
            .unwrap_or(0);
        for edge in edges.iter().take(20) {
            println!(
                "    {:<width$} {} {}  {}",
                edge.from,
                "\u{2192}".truecolor(255, 138, 0),
                edge.to.bold(),
                format!("({})", edge.count).dimmed(),
                width = width
            );
        }
    }

    println!();
    ui::footer(
        "Export",
        "blink map --format graph  (Mermaid)  ·  blink export",
    );
}

fn render_markdown(graph: &blink_context::ContextGraph) {
    println!("# Architecture — {}\n", graph.project.name);
    println!("## Areas\n");
    println!("| Area | Files | Symbols |");
    println!("| --- | ---: | ---: |");
    for area in graph.ranked_areas() {
        println!("| `{}` | {} | {} |", area.path, area.files, area.symbols);
    }
    let edges = graph.area_edges();
    if !edges.is_empty() {
        println!("\n## Area dependencies\n");
        for edge in edges {
            println!("- `{}` → `{}` ({})", edge.from, edge.to, edge.count);
        }
    }
}

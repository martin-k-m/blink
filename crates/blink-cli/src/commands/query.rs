use anyhow::Result;
use colored::Colorize;

use crate::cli::QueryArgs;
use crate::commands::context;
use crate::ui;

pub fn run(args: QueryArgs) -> Result<()> {
    let spinner = ui::spinner("Building context...");
    let graph = context::build(&args.path)?;
    spinner.finish_and_clear();

    let results = blink_query::query(&graph, &args.query, args.limit);

    if args.json {
        let json = serde_json::json!({
            "query": results.query,
            "terms": results.terms,
            "areas": results.areas.iter().map(|m| serde_json::json!({
                "path": m.name, "detail": m.detail, "score": m.score,
            })).collect::<Vec<_>>(),
            "files": results.files.iter().map(|m| serde_json::json!({
                "path": m.name, "detail": m.detail, "score": m.score,
            })).collect::<Vec<_>>(),
            "symbols": results.symbols.iter().map(|m| serde_json::json!({
                "name": m.name, "kind": m.kind, "file": m.file, "line": m.line, "score": m.score,
            })).collect::<Vec<_>>(),
            "dependencies": results.dependencies.iter().map(|m| serde_json::json!({
                "name": m.name, "detail": m.detail, "score": m.score,
            })).collect::<Vec<_>>(),
            "commands": results.commands.iter().map(|m| serde_json::json!({
                "name": m.name, "command": m.detail, "score": m.score,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner(&format!("Blink Query \u{2014} {}", results.query));

    if results.terms.is_empty() {
        println!();
        ui::field("", "no searchable terms in the query");
        return Ok(());
    }
    ui::field("Searching for", results.terms.join(", "));

    if results.is_empty() {
        println!();
        println!(
            "    {} nothing in the project's context matched.",
            "\u{2192}".truecolor(255, 45, 141)
        );
        println!(
            "    {}",
            "Try a broader term, or `blink map` for an overview.".dimmed()
        );
        return Ok(());
    }

    section("Areas", &results.areas);
    section("Files", &results.files);

    if !results.symbols.is_empty() {
        println!();
        println!("  {}", "Symbols".bold());
        for m in &results.symbols {
            println!(
                "    {} {}  {}",
                m.kind.dimmed(),
                m.name.bold(),
                format!("{}:{}", m.file, m.line).dimmed()
            );
        }
    }

    section("Dependencies", &results.dependencies);
    section("Commands", &results.commands);

    println!();
    ui::footer(
        "Found",
        format!("{} result(s) across the context graph", results.total()),
    );
    Ok(())
}

fn section(title: &str, matches: &[blink_query::Match]) {
    if matches.is_empty() {
        return;
    }
    println!();
    println!("  {}", title.bold());
    for m in matches {
        println!("    {}  {}", m.name.bold(), m.detail.dimmed());
    }
}

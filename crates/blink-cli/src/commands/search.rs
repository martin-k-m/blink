use anyhow::Result;
use blink_analyzer::format_bytes;
use colored::Colorize;

use crate::cli::{SearchArgs, SymbolsArgs};
use crate::indexing::ensure_index;
use crate::ui;

pub fn search(args: SearchArgs) -> Result<()> {
    let index = ensure_index(&args.path)?;

    if args.symbols {
        let hits = index.search_symbols(Some(&args.query));
        if args.json {
            let json: Vec<_> = hits
                .iter()
                .map(|(path, sym)| {
                    serde_json::json!({
                        "name": sym.name,
                        "kind": sym.kind.to_string(),
                        "path": path,
                        "line": sym.line,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
            return Ok(());
        }

        ui::banner(&format!("Search: symbols matching \"{}\"", args.query));
        if hits.is_empty() {
            println!("\n  No matching symbols.\n");
            return Ok(());
        }
        println!();
        for (path, sym) in &hits {
            println!(
                "  {} {}  {}",
                sym.kind.to_string().dimmed(),
                sym.name.bold(),
                format!("{path}:{}", sym.line).dimmed()
            );
        }
        ui::footer("Matches", hits.len());
        return Ok(());
    }

    let hits = index.search_paths(&args.query);
    if args.json {
        let json: Vec<_> = hits
            .iter()
            .map(|f| {
                serde_json::json!({
                    "path": f.path,
                    "bytes": f.size,
                    "lines": f.lines,
                    "language": f.lang.map(|l| l.to_string()),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner(&format!("Search: files matching \"{}\"", args.query));
    if hits.is_empty() {
        println!("\n  No matching files.\n");
        return Ok(());
    }
    println!();
    for file in &hits {
        println!("  {}  {}", file.path, format_bytes(file.size).dimmed());
    }
    ui::footer("Matches", hits.len());
    Ok(())
}

pub fn symbols(args: SymbolsArgs) -> Result<()> {
    let index = ensure_index(&args.path)?;
    let hits = index.search_symbols(args.filter.as_deref());

    if args.json {
        let json: Vec<_> = hits
            .iter()
            .map(|(path, sym)| {
                serde_json::json!({
                    "name": sym.name,
                    "kind": sym.kind.to_string(),
                    "path": path,
                    "line": sym.line,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Symbols");
    if let Some(filter) = &args.filter {
        ui::field("Filter", filter);
    }
    if hits.is_empty() {
        println!("\n  No symbols found.\n");
        return Ok(());
    }
    println!();
    for (path, sym) in &hits {
        println!(
            "  {} {}  {}",
            sym.kind.to_string().dimmed(),
            sym.name.bold(),
            format!("{path}:{}", sym.line).dimmed()
        );
    }
    ui::footer("Symbols", hits.len());
    Ok(())
}

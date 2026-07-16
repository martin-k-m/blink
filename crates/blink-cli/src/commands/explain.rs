use anyhow::{bail, Result};
use colored::Colorize;

use crate::cli::ExplainArgs;
use crate::commands::context::{self, human_size};
use crate::ui;

pub fn run(args: ExplainArgs) -> Result<()> {
    let spinner = ui::spinner("Building context...");
    let graph = context::build(&args.path)?;
    spinner.finish_and_clear();

    let target = normalize(&args.file);
    let Some(ex) = graph.explain(&target) else {
        bail!(
            "'{}' isn't a file in this project's index.\n  \
             Give a project-relative path (e.g. src/main.rs), or run \
             `blink search {}` to find it.",
            args.file,
            leaf(&target)
        );
    };

    if args.json {
        let json = serde_json::json!({
            "file": ex.path,
            "area": ex.area,
            "language": ex.lang,
            "lines": ex.lines,
            "size_bytes": ex.size_bytes,
            "doc": ex.doc,
            "symbols": ex.symbols.iter().map(|s| serde_json::json!({
                "name": s.name, "kind": s.kind, "line": s.line,
            })).collect::<Vec<_>>(),
            "imports_internal": ex.references,
            "imports_external": ex.external,
            "used_by": ex.dependents,
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner(&format!("Blink Explain \u{2014} {}", ex.path));

    ui::field("Area", &ex.area);
    if let Some(lang) = &ex.lang {
        ui::field("Language", lang);
    }
    ui::field("Lines", ui::format_count(ex.lines));
    ui::field("Size", human_size(ex.size_bytes));

    // The file's own documentation, verbatim — never invented.
    if let Some(doc) = &ex.doc {
        println!();
        println!("  {}", "Documentation".bold());
        for line in doc.lines().take(6) {
            println!("    {}", line.trim());
        }
    }

    // What it declares.
    println!();
    println!("  {}", "Defines".bold());
    if ex.symbols.is_empty() {
        println!("    {}", "no top-level symbols found".dimmed());
    } else {
        for sym in ex.symbols.iter().take(30) {
            println!(
                "    {} {}  {}",
                sym.kind.dimmed(),
                sym.name.bold(),
                format!("(line {})", sym.line).dimmed()
            );
        }
        if ex.symbols.len() > 30 {
            println!(
                "    {}",
                format!("... and {} more", ex.symbols.len() - 30).dimmed()
            );
        }
    }

    // What it imports.
    if !ex.references.is_empty() {
        println!();
        println!("  {}", "Imports (this project)".bold());
        for r in &ex.references {
            println!("    {r}");
        }
    }
    if !ex.external.is_empty() {
        println!();
        println!("  {}", "Imports (external)".bold());
        println!("    {}", ex.external.join(", "));
    }

    // What depends on it.
    println!();
    println!("  {}", "Used by".bold());
    if ex.dependents.is_empty() {
        println!(
            "    {}",
            "no other indexed file references this one".dimmed()
        );
    } else {
        for d in &ex.dependents {
            println!("    {d}");
        }
    }

    println!();
    ui::footer(
        "Note",
        "Every line above is read from the file or the index — nothing is inferred.",
    );
    Ok(())
}

/// Normalize a user-supplied file path to the index's `/`-separated,
/// project-relative form.
fn normalize(file: &str) -> String {
    let f = file.replace('\\', "/");
    f.strip_prefix("./").unwrap_or(&f).to_string()
}

fn leaf(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

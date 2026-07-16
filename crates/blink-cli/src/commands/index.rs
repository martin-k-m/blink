use anyhow::{Context, Result};
use blink_analyzer::format_bytes;
use blink_index::Index;

use crate::cli::{FormatArgs, IndexArgs};
use crate::ui;

pub fn run(args: IndexArgs) -> Result<()> {
    let spinner = ui::spinner(if args.rebuild {
        "Rebuilding index..."
    } else {
        "Refreshing index..."
    });

    let (index, stats) = if args.rebuild {
        Index::build(&args.path)
    } else {
        Index::refresh(&args.path)
    }
    .with_context(|| format!("could not index {}", args.path.display()))?;
    index
        .save()
        .with_context(|| format!("could not write index for {}", args.path.display()))?;
    spinner.finish_and_clear();

    ui::banner("Blink Index");
    ui::field("Files indexed", ui::format_count(index.file_count()));
    ui::field("Symbols", ui::format_count(index.symbol_count()));
    ui::field("Lines", ui::format_count(index.total_lines()));
    ui::field("Index size", format_bytes(index.total_size()));

    println!();
    if stats.changed() {
        ui::step(format!(
            "{} added, {} updated, {} removed, {} unchanged",
            stats.added, stats.updated, stats.removed, stats.unchanged
        ));
        ui::step(format!(
            "reprocessed {} of {} files",
            stats.reprocessed(),
            index.file_count()
        ));
    } else {
        ui::step(format!("up to date ({} files unchanged)", stats.unchanged));
    }

    println!();
    Ok(())
}

pub fn status(args: FormatArgs) -> Result<()> {
    match Index::load(&args.path) {
        Some(index) => {
            if args.json {
                let json = serde_json::json!({
                    "indexed": true,
                    "files": index.file_count(),
                    "symbols": index.symbol_count(),
                    "lines": index.total_lines(),
                    "bytes": index.total_size(),
                    "generated_at": index.generated_at,
                    "languages": index
                        .language_breakdown()
                        .iter()
                        .map(|(lang, (files, lines))| {
                            serde_json::json!({
                                "language": lang.to_string(),
                                "files": files,
                                "lines": lines,
                            })
                        })
                        .collect::<Vec<_>>(),
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
                return Ok(());
            }

            ui::banner("Blink Index Status");
            ui::field("Status", "indexed");
            ui::field("Files", ui::format_count(index.file_count()));
            ui::field("Symbols", ui::format_count(index.symbol_count()));
            ui::field("Lines", ui::format_count(index.total_lines()));
            ui::field("Index size", format_bytes(index.total_size()));

            let langs = index.language_breakdown();
            if !langs.is_empty() {
                println!();
                println!("  Languages");
                for (lang, (files, lines)) in langs {
                    ui::field(
                        &lang.to_string(),
                        format!(
                            "{} files, {} lines",
                            ui::format_count(files),
                            ui::format_count(lines)
                        ),
                    );
                }
            }
            println!();
        }
        None => {
            if args.json {
                println!("{}", serde_json::json!({ "indexed": false }));
                return Ok(());
            }
            ui::banner("Blink Index Status");
            ui::field("Status", "not indexed");
            println!();
            ui::suggestion("Run `blink index` to build the index.");
            println!();
        }
    }
    Ok(())
}

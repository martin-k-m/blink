use anyhow::{Context as _, Result};

use crate::cli::ExportArgs;
use crate::commands::context;
use crate::ui;

pub fn run(args: ExportArgs) -> Result<()> {
    let format = args
        .format
        .parse::<blink_export::ExportFormat>()
        .map_err(anyhow::Error::msg)?;

    let spinner = ui::spinner("Building context...");
    let graph = context::build(&args.path)?;
    spinner.finish_and_clear();

    let rendered = blink_export::export(&graph, format);

    match &args.output {
        Some(path) => {
            std::fs::write(path, &rendered)
                .with_context(|| format!("could not write {}", path.display()))?;
            ui::banner("Blink Export");
            ui::field("Format", format.to_string());
            ui::field("Wrote", path.display().to_string());
            ui::field(
                "Contents",
                format!(
                    "{} areas · {} files · {} references",
                    graph.areas.len(),
                    graph.files.len(),
                    graph.references.len()
                ),
            );
        }
        None => {
            // Straight to stdout so it pipes cleanly (`blink export | jq`).
            print!("{rendered}");
        }
    }
    Ok(())
}

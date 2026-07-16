use anyhow::{Context, Result};
use blink_analyzer::compute_health;
use blink_core::ProjectDetector;

use crate::analysis::analyze_cached;
use crate::cli::ReportArgs;
use crate::ui;

pub fn run(args: ReportArgs) -> Result<()> {
    let spinner = (args.output.is_some() || !args.json).then(|| ui::spinner("Building report..."));

    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;
    let analysis = analyze_cached(&project, &args.path, args.online).report;
    let health = compute_health(&analysis, &args.path);

    if let Some(spinner) = &spinner {
        spinner.finish_and_clear();
    }

    let rendered = if args.markdown {
        blink_report::render_markdown(&project, &analysis, &health)
    } else if args.html {
        blink_report::render_html(&project, &analysis, &health)
    } else {
        let json_report = blink_report::to_json_report(&project, &analysis);
        serde_json::to_string_pretty(&json_report)?
    };

    match &args.output {
        Some(path) => {
            std::fs::write(path, &rendered)
                .with_context(|| format!("could not write report to {}", path.display()))?;
            ui::banner("Blink Report");
            println!();
            ui::step(format!("Written to {}", path.display()));
            println!();
        }
        None => {
            print!("{rendered}");
            if !rendered.ends_with('\n') {
                println!();
            }
        }
    }

    Ok(())
}

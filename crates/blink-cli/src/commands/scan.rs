use std::time::Instant;

use anyhow::{Context, Result};
use blink_core::ProjectDetector;

use crate::cli::ScanArgs;
use crate::ui;

pub fn run(args: ScanArgs) -> Result<()> {
    let start = Instant::now();
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("no recognizable project in {}", args.path.display()))?;
    let elapsed = start.elapsed();

    ui::banner("Blink Project Scanner");
    ui::field("Project", &project.name);
    ui::field("Framework", project.framework);
    ui::field("Language", project.language);
    ui::field("Package manager", project.package_manager);
    ui::field("Files", ui::format_count(project.file_count));
    ui::field("Dependencies", ui::format_count(project.dependency_count()));

    ui::footer("Scan completed:", format!("{}ms", elapsed.as_millis()));
    Ok(())
}

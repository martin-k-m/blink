use std::time::Instant;

use anyhow::{Context, Result};
use blink_analyzer::Analyzer;
use blink_cache::{AnalysisCache, Cache};
use blink_core::ProjectDetector;
use colored::Colorize;

use crate::cli::BenchmarkArgs;
use crate::ui;

pub fn run(args: BenchmarkArgs) -> Result<()> {
    ui::banner("Blink Benchmark");
    println!();
    println!(
        "  {}",
        "Every number below is measured on this run, against this project — not a fixed or hypothetical value.".dimmed()
    );

    let startup_ms = measure_startup(args.runs)?;

    let scan_start = Instant::now();
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not benchmark {}", args.path.display()))?;
    let scan_ms = scan_start.elapsed().as_millis();

    let cold_start = Instant::now();
    let cold_report = Analyzer::new().analyze(&project, &args.path);
    let cold_ms = cold_start.elapsed().as_millis();

    let cached_ms = measure_cached_analysis(&args.path, &cold_report);

    println!();
    ui::field(
        "Startup",
        format!("{startup_ms}ms (min of {} runs)", args.runs),
    );
    ui::field("Scan", format!("{scan_ms}ms"));
    ui::field("Cold Analysis", format!("{cold_ms}ms"));
    match cached_ms {
        Some(ms) => ui::field("Cached Analysis", format!("{ms}ms")),
        None => ui::field(
            "Cached Analysis",
            "unavailable (couldn't open the global cache)",
        ),
    }

    println!();
    Ok(())
}

/// Spawn a fresh `blink --version` process `runs` times and return the
/// minimum wall-clock time — a real, externally-measured process-launch
/// cost (parsing args in-process would only measure a few microseconds of
/// work after the OS has already paid to start the process).
fn measure_startup(runs: usize) -> Result<u128> {
    let exe = std::env::current_exe().context("could not locate the running blink executable")?;
    let runs = runs.max(1);

    let mut best: Option<u128> = None;
    for _ in 0..runs {
        let start = Instant::now();
        std::process::Command::new(&exe)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .status()
            .context("could not launch blink for a startup measurement")?;
        let elapsed = start.elapsed().as_millis();
        best = Some(best.map_or(elapsed, |b: u128| b.min(elapsed)));
    }
    Ok(best.unwrap_or(0))
}

/// Warm the global analysis cache with an already-computed report (so we
/// don't pay for the analysis twice just to benchmark the cache), then
/// measure a genuine cache-hit lookup.
fn measure_cached_analysis(
    path: &std::path::Path,
    report: &blink_analyzer::AnalysisReport,
) -> Option<u128> {
    let cache = AnalysisCache::open().ok()?;
    let snapshot = Cache::scan(path);
    cache.set(path, snapshot, report).ok()?;

    let start = Instant::now();
    let _: blink_analyzer::AnalysisReport = cache.get(path, &Cache::scan(path))?;
    Some(start.elapsed().as_millis())
}
